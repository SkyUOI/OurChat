//! Database

pub mod file_storage;

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use config::File;
use migration::MigratorTrait;
use serde::{Deserialize, Serialize};
use static_keys::{define_static_key_false, static_branch_likely, static_branch_unlikely};

#[derive(Debug, Deserialize, Serialize)]
struct MysqlDbCfg {
    host: String,
    user: String,
    db: String,
    port: usize,
    passwd: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SqliteDbCfg {
    path: PathBuf,
}

impl SqliteDbCfg {
    fn convert_to_abs_path(&mut self, basepath: &Path) -> anyhow::Result<()> {
        self.path = base::resolve_relative_path(basepath, &self.path)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, strum::EnumString)]
pub enum DbType {
    #[serde(rename = "mysql")]
    #[strum(serialize = "mysql")]
    MySql,
    #[serde(rename = "sqlite")]
    #[strum(serialize = "sqlite")]
    Sqlite,
}

impl Default for DbType {
    fn default() -> Self {
        Self::MySql
    }
}

pub static DB_TYPE: OnceLock<DbType> = OnceLock::new();

define_static_key_false!(DB_INIT);
pub static MYSQL_TYPE: static_keys::StaticFalseKey = static_keys::new_static_false_key();
pub static SQLITE_TYPE: static_keys::StaticFalseKey = static_keys::new_static_false_key();

/// 初始化数据库层
pub fn init_db_system(db_type: DbType) {
    tracing::info!("Init db system");
    DB_TYPE.get_or_init(|| db_type.clone());
    tracing::info!("db type: {:?}", DB_TYPE.get().unwrap());
    if static_branch_unlikely!(DB_INIT) {
        tracing::error!("Init db system twice");
        panic!("Init db system twice");
    } else {
        unsafe { DB_INIT.enable() }
        match db_type {
            DbType::MySql => unsafe { MYSQL_TYPE.enable() },
            DbType::Sqlite => unsafe { SQLITE_TYPE.enable() },
        }
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
pub fn get_db_url(path: &Path, basepath: &Path) -> anyhow::Result<String> {
    let db_type = get_db_type();
    match db_type {
        DbType::MySql => {
            let mut cfg = config::Config::builder()
                .add_source(config::File::with_name(path.to_str().unwrap()))
                .build()?;
            let cfg: MysqlDbCfg = cfg.try_deserialize()?;
            let path = format!(
                "mysql://{}:{}@{}:{}/{}",
                cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
            );
            Ok(path)
        }
        DbType::Sqlite => {
            let mut cfg = config::Config::builder()
                .add_source(config::File::with_name(path.to_str().unwrap()))
                .build()?;
            let mut cfg: SqliteDbCfg = cfg.try_deserialize()?;
            cfg.convert_to_abs_path(basepath)?;
            Ok(format!("sqlite://{}", cfg.path.display()))
        }
    }
}

pub async fn try_create_sqliet_db(url: &str) -> anyhow::Result<()> {
    use sqlx::{Sqlite, migrate::MigrateDatabase};
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

pub async fn try_create_mysql_db(url: &str) -> anyhow::Result<()> {
    use sqlx::{MySql, migrate::MigrateDatabase};
    if !MySql::database_exists(url).await.unwrap_or(false) {
        tracing::info!("Creating mysql database");
        match MySql::create_database(url).await {
            Ok(_) => {
                tracing::info!("Created mysql database {}", url);
            }
            Err(e) => {
                tracing::error!("Failed to create mysql database: {}", e);
                anyhow::bail!("Failed to create mysql database: {}", e);
            }
        }
    }
    Ok(())
}

/// 根据url连接数据库
pub async fn connect_to_db(url: &str) -> anyhow::Result<sea_orm::DatabaseConnection> {
    let db_type = get_db_type();
    match db_type {
        DbType::MySql => {
            try_create_mysql_db(url).await?;
        }
        DbType::Sqlite => {
            try_create_sqliet_db(url).await?;
        }
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
pub fn get_redis_url(path: &Path) -> anyhow::Result<String> {
    let cfg = config::Config::builder()
        .add_source(File::with_name(path.to_str().unwrap()))
        .build()?;
    let cfg: RedisCfg = cfg.try_deserialize()?;
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
