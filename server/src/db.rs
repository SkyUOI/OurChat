//! Database

pub mod user;

use std::sync::OnceLock;

use migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use static_keys::{define_static_key_true, static_branch_likely, static_branch_unlikely};

#[derive(Debug, Deserialize, Serialize)]
struct DbCfg {
    host: String,
    user: String,
    db: String,
    port: usize,
    passwd: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DbType {
    #[serde(rename = "mysql")]
    Mysql,
    #[serde(rename = "sqlite")]
    Sqlite,
}

impl Default for DbType {
    fn default() -> Self {
        Self::Mysql
    }
}

pub const DB_TYPE: OnceLock<DbType> = OnceLock::new();
define_static_key_true!(DB_INIT);

/// 初始化数据库层
pub fn init_db_system(db_type: DbType) {
    DB_TYPE.get_or_init(|| db_type);
    if static_branch_unlikely!(DB_INIT) {
        tracing::error!("Init db sysytem twice");
        panic!("Init db sysytem twice");
    } else {
        unsafe { DB_INIT.enable() }
    }
}

pub fn get_db_type() -> DbType {
    if static_branch_likely!(DB_INIT) {
        DB_TYPE.get().unwrap().clone()
    } else {
        tracing::error!("Db system has not been inited");
        panic!("Db system has not been inited");
    }
}

/// 根据配置文件生成连接数据库的url
pub fn get_db_url(path: &str) -> anyhow::Result<String> {
    let db_type = get_db_type();
    let json = std::fs::read_to_string(path)?;
    let cfg: DbCfg = serde_json::from_str(&json)?;
    match db_type {
        DbType::Mysql => {
            let path = format!(
                "mysql://{}:{}@{}:{}/{}",
                cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
            );
            Ok(path)
        }
        DbType::Sqlite => {
            todo!()
        }
    }
}

/// 根据url连接数据库
pub async fn connect_to_db(url: &str) -> anyhow::Result<sea_orm::DatabaseConnection> {
    tracing::info!("Connecting to {}", url);
    Ok(sea_orm::Database::connect(url).await?)
}

/// 初始化数据库并运行迁移
pub async fn init_db(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    migration::Migrator::up(db, None).await?;
    tracing::info!("Ran all migrations of databases");
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
