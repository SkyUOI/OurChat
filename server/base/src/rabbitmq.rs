pub mod http_server;

use config::File;
use deadpool_lapin::lapin::types::FieldTable;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

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

    pub async fn build(&self) -> anyhow::Result<deadpool_lapin::Pool> {
        let url = self.url();
        let rmq_pool_cfg = deadpool_lapin::Config {
            url: Some(url),
            ..Default::default()
        };
        match tokio::time::timeout(
            Duration::from_secs(10),
            tokio::spawn(
                async move { rmq_pool_cfg.create_pool(Some(deadpool_lapin::Runtime::Tokio1)) },
            ),
        )
        .await
        {
            Ok(Ok(pool)) => Ok(pool?),
            Ok(Err(e)) => Err(anyhow::anyhow!("Failed to create rabbitmq pool:{e}")),
            Err(_) => Err(anyhow::anyhow!("Failed to create rabbitmq pool: timeout")),
        }
    }
}
