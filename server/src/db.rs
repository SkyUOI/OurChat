//! Database

pub mod user;

use migration::MigratorTrait;
use sea_orm::{ConnectionTrait, Statement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct DbCfg {
    host: String,
    user: String,
    db: String,
    port: usize,
    passwd: String,
}

/// 根据配置文件生成连接数据库的url
pub fn get_db_url(path: &str) -> anyhow::Result<String> {
    let json = std::fs::read_to_string(path)?;
    let cfg: DbCfg = serde_json::from_str(&json)?;
    let path = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
    );
    Ok(path)
}

/// 根据url连接数据库
pub async fn connect_to_db(url: &str) -> anyhow::Result<sea_orm::DatabaseConnection> {
    tracing::info!("Connecting to {}", url);
    Ok(sea_orm::Database::connect(url).await?)
}

/// 初始化数据库并运行迁移
pub async fn init_db(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    migration::Migrator::up(db, None).await?;
    tracing::info!("Runned all migrations of databases");
    tracing::info!("Initialized database");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct RedisCfg {
    host: String,
    port: usize,
    passwd: String,
    user: String,
}

/// 根据配置文件生成连接redis的url
pub fn get_redis_url(path: &str) -> anyhow::Result<String> {
    let json = std::fs::read_to_string(path)?;
    let cfg: RedisCfg = serde_json::from_str(&json)?;
    let path = format!(
        "redis://{}:{}@{}:{}/",
        cfg.user, cfg.passwd, cfg.host, cfg.port
    );
    Ok(path)
}

/// 根据url连接redis
pub async fn connect_to_redis(url: &str) -> anyhow::Result<redis::Client> {
    let client = redis::Client::open(url)?;
    Ok(client)
}
