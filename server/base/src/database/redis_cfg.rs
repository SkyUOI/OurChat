use crate::setting::Setting;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisCfg {
    host: String,
    port: usize,
    passwd: String,
    user: String,
}

impl RedisCfg {
    /// Generate the url for connecting to redis according to the configuration file
    pub fn get_redis_url(&self) -> anyhow::Result<String> {
        let path = format!(
            "redis://{}:{}@{}:{}/",
            self.user, self.passwd, self.host, self.port
        );
        Ok(path)
    }
}

impl Setting for RedisCfg {}

impl crate::setting::PathConvert for RedisCfg {
    fn convert_to_abs_path(&mut self, _full_basepath: &std::path::Path) -> anyhow::Result<()> {
        Ok(())
    }
}

/// connect to redis database according to url
pub async fn connect_to_redis(url: &str) -> anyhow::Result<::redis::aio::ConnectionManager> {
    let client = ::redis::Client::open(url)?;
    // ConnectionManager will handle the connection and auto-reconnect
    Ok(::redis::aio::ConnectionManager::new(client).await?)
}
