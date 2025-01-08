#![feature(duration_constructors)]

use crate::httpserver::HttpServer;
use base::database::DbPool;
use base::email_client::{EmailCfg, EmailSender};
use base::rabbitmq::RabbitMQCfg;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
struct ArgParser {
    #[clap(short, long, help = "Path to the config file")]
    config: Option<PathBuf>,
}

pub struct Launcher {
    pub config: Config,
    pub email_client: Option<EmailClientType>,
    pub tcplistener: Option<tokio::net::TcpListener>,
    pub rabbitmq_cfg: RabbitMQCfg,
}

pub type EmailClientType = Box<dyn EmailSender>;

impl Launcher {
    pub fn get_config() -> anyhow::Result<Config> {
        let parser = ArgParser::parse();
        let config_file_path =
            parser
                .config
                .unwrap_or_else(|| match std::env::var("OURCHAT_HTTP_CONFIG_FILE") {
                    Ok(path) => PathBuf::from(path),
                    Err(_) => {
                        eprintln!("Please specify the config file path");
                        std::process::exit(1);
                    }
                });
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
        Ok(Self {
            config: cfg,
            email_client,
            tcplistener: Some(http_listener),
            rabbitmq_cfg,
        })
    }

    pub async fn build() -> anyhow::Result<Self> {
        let cfg = Self::get_config()?;
        Self::build_from_config(cfg).await
    }

    pub async fn run_forever(&mut self) -> anyhow::Result<()> {
        let mut server = HttpServer::new();
        let redis_cfg = base::database::redis::RedisCfg::build_from_path(&self.config.rediscfg)?;
        let postgres_cfg =
            base::database::postgres::PostgresDbCfg::build_from_path(&self.config.dbcfg)?;
        let db_pool = DbPool::build(&postgres_cfg, &redis_cfg).await?;
        let rabbitmq_pool = self.rabbitmq_cfg.build()?;
        server
            .run_forever(
                self.tcplistener.take().unwrap(),
                self.email_client.take(),
                self.config.clone(),
                rabbitmq_pool.clone(),
                db_pool,
            )
            .await?;
        rabbitmq_pool.close();
        Ok(())
    }
}
