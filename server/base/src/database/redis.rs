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

/// connect to redis database according to url
pub async fn connect_to_redis(url: &str) -> anyhow::Result<deadpool_redis::Pool> {
    let cfg = deadpool_redis::Config::from_url(url);
    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    #[cfg(debug_assertions)]
    {
        // try getting a connection
        pool.get().await?;
    }
    Ok(pool)
}
