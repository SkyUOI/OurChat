#![feature(duration_constructors)]

use crate::httpserver::HttpServer;
use base::database::DbPool;
use base::email_client::{EmailCfg, EmailSender};
use base::rabbitmq::RabbitMQCfg;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod httpserver;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
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
}

impl Config {
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

pub struct Launcher {
    pub config: Config,
    pub email_client: Option<EmailClientType>,
    pub tcplistener: Option<tokio::net::TcpListener>,
    pub rabbitmq_cfg: RabbitMQCfg,
    pub started_notify: Arc<tokio::sync::Notify>,
}

pub type EmailClientType = Box<dyn EmailSender>;

impl Launcher {
    pub fn get_config(parser: Option<ArgParser>) -> anyhow::Result<Config> {
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
        let mut cfg: Config = cfg.try_deserialize()?;
        cfg.fix_paths(&config_file_path)?;
        Ok(cfg)
    }

    pub async fn build_from_config(mut cfg: Config) -> anyhow::Result<Self> {
        base::log::logger_init(false, None, std::io::stdout, "http_server");
        let email_client: Option<Box<dyn EmailSender>> = match &cfg.email_cfg {
            Some(email_cfg) => {
                let email_cfg = EmailCfg::build_from_path(email_cfg)?;
                let email_client = email_cfg.build_email_client()?;
                Some(Box::new(email_client))
            }
            None => None,
        };
        let http_listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &cfg.ip, cfg.port)).await?;
        // deal with port 0
        cfg.port = http_listener.local_addr()?.port();
        let rabbitmq_cfg = RabbitMQCfg::build_from_path(&cfg.rabbitmq_cfg)?;
        let started_notify = Arc::new(tokio::sync::Notify::new());
        Ok(Self {
            config: cfg,
            email_client,
            tcplistener: Some(http_listener),
            rabbitmq_cfg,
            started_notify,
        })
    }

    pub async fn build() -> anyhow::Result<Self> {
        let cfg = Self::get_config(Some(ArgParser::parse()))?;
        Self::build_from_config(cfg).await
    }

    pub async fn run_forever(&mut self) -> anyhow::Result<()> {
        let mut server = HttpServer::new();
        let redis_cfg = base::database::redis::RedisCfg::build_from_path(&self.config.rediscfg)?;
        let postgres_cfg =
            base::database::postgres::PostgresDbCfg::build_from_path(&self.config.dbcfg)?;
        let db_pool = DbPool::build(&postgres_cfg, &redis_cfg).await?;
        tracing::info!("Get database pool");
        let rabbitmq_pool = self.rabbitmq_cfg.build().await?;
        tracing::info!("Connected to RabbitMQ");
        tracing::info!("Starting http server");
        let handle = server
            .run_forever(
                self.tcplistener.take().unwrap(),
                self.email_client.take(),
                self.config.clone(),
                rabbitmq_pool.clone(),
                db_pool,
            )
            .await?;
        let http_server = tokio::spawn(handle);
        tracing::info!("Started http server");
        tracing::info!("Sending started notification");
        self.started_notify.notify_waiters();
        tracing::info!("Http Server started");
        http_server.await??;
        rabbitmq_pool.close();
        Ok(())
    }
}

#[ctor::ctor]
fn init() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
