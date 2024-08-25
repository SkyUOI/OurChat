//! Database

pub mod user;

use std::sync::OnceLock;

use migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use static_keys::{define_static_key_false, static_branch_likely, static_branch_unlikely};

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

pub static DB_TYPE: OnceLock<DbType> = OnceLock::new();
define_static_key_false!(DB_INIT);

/// 初始化数据库层
pub fn init_db_system(db_type: DbType) {
    tracing::info!("Init db system");
    DB_TYPE.get_or_init(|| db_type);
    tracing::info!("db type: {:?}", DB_TYPE.get().unwrap());
    if static_branch_unlikely!(DB_INIT) {
        tracing::error!("Init db system twice");
        panic!("Init db system twice");
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
        DbType::Sqlite => Ok("sqlite://ourchat.db".to_owned()),
    }
}

pub async fn create_sqliet_db(url: &str) -> anyhow::Result<()> {
    use sqlx::{migrate::MigrateDatabase, Sqlite};
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        tracing::info!("Creating sqlite database {}", url);
        match Sqlite::create_database(url).await {
            Ok(_) => {
                tracing::info!("Created sqlite database {}", url);
            }
            Err(e) => {
                tracing::error!("Failed to create sqlite database: {}", e);
                anyhow::bail!("Failed to create sqlite database: {}", e);
            }
        }
    }

    Ok(())
}

/// 根据url连接数据库
pub async fn connect_to_db(url: &str) -> anyhow::Result<sea_orm::DatabaseConnection> {
    let db_type = get_db_type();
    if let DbType::Sqlite = db_type {
        create_sqliet_db(url).await?;
    }
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
