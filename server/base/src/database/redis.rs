use config::File;
use serde::{Deserialize, Serialize};
use std::path::Path;

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

    pub fn build_from_path(path: &Path) -> anyhow::Result<Self> {
        let cfg = config::Config::builder()
            .add_source(File::with_name(path.to_str().unwrap()))
            .build()?;
        let cfg: RedisCfg = cfg.try_deserialize()?;
        Ok(cfg)
    }
}

/// connect to redis database according to url
pub async fn connect_to_redis(url: &str) -> anyhow::Result<deadpool_redis::Pool> {
    let cfg = deadpool_redis::Config::from_url(url);
    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    Ok(pool)
}
