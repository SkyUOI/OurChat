#![feature(duration_constructors)]

use crate::httpserver::HttpServer;
use base::database::DbPool;
use base::email_client::{EmailCfg, EmailSender};
use base::rabbitmq::RabbitMQCfg;
use base::shutdown::ShutdownSdr;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod httpserver;
pub mod matrix;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MainCfg {
    #[serde(default = "base::consts::default_ip")]
    pub ip: String,
    #[serde(default = "base::consts::default_http_port")]
    pub port: u16,
    #[serde(default = "base::consts::default_ssl")]
    pub ssl: bool,
    pub rediscfg: PathBuf,
    pub dbcfg: PathBuf,
    pub email_cfg: Option<PathBuf>,
    pub rabbitmq_cfg: PathBuf,
    pub logo_path: PathBuf,
    #[serde(default = "base::consts::default_http_run_migration")]
    pub run_migration: bool,
    #[serde(default = "base::consts::default_enable_matrix")]
    pub enable_matrix: bool,
}

impl MainCfg {
    pub fn protocol_http(&self) -> &'static str {
        if self.ssl { "https" } else { "http" }
    }

    pub fn fix_paths(&mut self, base_path: &Path) -> anyhow::Result<()> {
        let full_basepath = base_path.parent().unwrap().canonicalize()?;
        self.rediscfg = base::resolve_relative_path(&full_basepath, Path::new(&self.rediscfg))?;
        self.dbcfg = base::resolve_relative_path(&full_basepath, Path::new(&self.dbcfg))?;
        self.rabbitmq_cfg =
            base::resolve_relative_path(&full_basepath, Path::new(&self.rabbitmq_cfg))?;
        self.logo_path = base::resolve_relative_path(&full_basepath, Path::new(&self.logo_path))?;
        if let Some(email_cfg) = &self.email_cfg {
            self.email_cfg = Some(base::resolve_relative_path(
                &full_basepath,
                Path::new(email_cfg),
            )?);
        }
        Ok(())
    }
}

#[derive(clap::Parser)]
pub struct ArgParser {
    #[clap(short, long, help = "Path to the config file")]
    config: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Cfg {
    pub main_cfg: MainCfg,
    pub rabbitmq_cfg: RabbitMQCfg,
    pub db_cfg: base::database::postgres::PostgresDbCfg,
    pub redis_cfg: base::database::redis::RedisCfg,
}

#[derive(Debug)]
pub struct Launcher {
    pub email_client: Option<EmailClientType>,
    pub tcplistener: Option<tokio::net::TcpListener>,
    pub started_notify: Arc<tokio::sync::Notify>,
    pub shared_data: Arc<Cfg>,
    pub abort_sender: ShutdownSdr,
}

pub type EmailClientType = Box<dyn EmailSender>;

impl Launcher {
    pub fn get_config(parser: Option<ArgParser>) -> anyhow::Result<Cfg> {
        let get_from_env = || match std::env::var("OURCHAT_HTTP_CONFIG_FILE") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                eprintln!("Please specify the config file path");
                std::process::exit(1);
            }
        };
        let config_file_path = match parser {
            Some(parser) => parser.config.unwrap_or_else(get_from_env),
            None => get_from_env(),
        };
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(config_file_path.to_str().unwrap()))
            .build()?;
        let mut cfg: MainCfg = cfg.try_deserialize()?;
        cfg.fix_paths(&config_file_path)?;
        let rabbitmq_cfg = RabbitMQCfg::build_from_path(&cfg.rabbitmq_cfg)?;
        let redis_cfg = base::database::redis::RedisCfg::build_from_path(&cfg.rediscfg)?;
        let postgres_cfg = base::database::postgres::PostgresDbCfg::build_from_path(&cfg.dbcfg)?;
        Ok(Cfg {
            main_cfg: cfg,
            rabbitmq_cfg,
            db_cfg: postgres_cfg,
            redis_cfg,
        })
    }

    pub async fn build_from_config(mut cfg: Cfg) -> anyhow::Result<Self> {
        base::log::logger_init(false, None, std::io::stdout, "http_server");
        let email_client: Option<Box<dyn EmailSender>> = match &cfg.main_cfg.email_cfg {
            Some(email_cfg) => {
                let email_cfg = EmailCfg::build_from_path(email_cfg)?;
                let email_client = email_cfg.build_email_client()?;
                Some(Box::new(email_client))
            }
            None => None,
        };
        let http_listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &cfg.main_cfg.ip, cfg.main_cfg.port))
                .await?;
        // deal with port 0
        cfg.main_cfg.port = http_listener.local_addr()?.port();
        let started_notify = Arc::new(tokio::sync::Notify::new());
        Ok(Self {
            email_client,
            tcplistener: Some(http_listener),
            started_notify,
            shared_data: Arc::new(cfg),
            abort_sender: ShutdownSdr::new(None),
        })
    }

    pub async fn build() -> anyhow::Result<Self> {
        let cfg = Self::get_config(Some(ArgParser::parse()))?;
        Self::build_from_config(cfg).await
    }

    pub async fn run_forever(&mut self) -> anyhow::Result<()> {
        let mut server = HttpServer::new();

        let db_pool = DbPool::build(
            &self.shared_data.db_cfg,
            &self.shared_data.redis_cfg,
            self.shared_data.main_cfg.run_migration,
        )
        .await?;
        tracing::info!("Get database pool");
        let rabbitmq_pool = self.shared_data.rabbitmq_cfg.build().await?;
        tracing::info!("Connected to RabbitMQ");
        tracing::info!("Starting http server");
        let tcplistener = self.tcplistener.take().unwrap();
        let rabbitmq_pool_clone = rabbitmq_pool.clone();
        let email_client = self.email_client.take();
        let main_cfg = self.shared_data.main_cfg.clone();
        let abort_sender_clone = self.abort_sender.clone();
        let http_server = tokio::spawn(async move {
            server
                .run_forever(
                    tcplistener,
                    email_client,
                    main_cfg,
                    rabbitmq_pool_clone,
                    db_pool,
                    abort_sender_clone,
                )
                .await
        });
        tracing::info!("Started http server");
        tracing::info!("Sending started notification");
        self.started_notify.notify_waiters();
        tracing::info!("Http Server started");
        http_server.await??;
        rabbitmq_pool.close();
        Ok(())
    }

    pub fn get_abort_handle(&self) -> ShutdownSdr {
        self.abort_sender.clone()
    }
}

#[ctor::ctor]
fn init() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
