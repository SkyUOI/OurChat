pub mod http_server;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::setting;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RabbitMQCfg {
    #[serde(default)]
    pub inherit: Option<String>,
    pub host: String,
    pub user: String,
    pub port: usize,
    pub passwd: String,
    pub vhost: String,
    pub manage_port: Option<usize>,
}

impl RabbitMQCfg {
    pub fn build_from_path(path: &Path) -> anyhow::Result<Self> {
        let cfg = setting::read_config_and_deserialize(path)?;
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
