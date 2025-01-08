pub mod http_server;

use config::File;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RabbitMQCfg {
    pub host: String,
    pub user: String,
    pub port: usize,
    pub passwd: String,
    pub vhost: String,
    pub manage_port: Option<usize>,
}

impl RabbitMQCfg {
    pub fn build_from_path(path: &Path) -> anyhow::Result<Self> {
        let cfg = config::Config::builder()
            .add_source(File::with_name(path.to_str().unwrap()))
            .build()?;
        let cfg: RabbitMQCfg = cfg.try_deserialize()?;
        Ok(cfg)
    }

    pub fn url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.user,
            self.passwd,
            self.host,
            self.port,
            urlencoding::encode(&self.vhost)
        )
    }

    pub fn url_without_vhost(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}",
            self.user, self.passwd, self.host, self.port
        )
    }

    pub fn manage_url(&self) -> Option<String> {
        self.manage_port.map(|port| {
            // FIXME: https
            format!(
                "http://{}:{}@{}:{}",
                self.user, self.passwd, self.host, port
            )
        })
    }

    pub fn build(&self) -> anyhow::Result<deadpool_lapin::Pool> {
        let url = self.url();
        let rmq_pool_cfg = deadpool_lapin::Config {
            url: Some(url),
            ..Default::default()
        };
        let pool = rmq_pool_cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1))?;
        Ok(pool)
    }
}
