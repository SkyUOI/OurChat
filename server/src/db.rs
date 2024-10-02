//! Database

pub mod file_storage;

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use config::File;
use migration::MigratorTrait;
use parking_lot::Once;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MysqlDbCfg {
    pub host: String,
    pub user: String,
    pub db: String,
    pub port: usize,
    pub passwd: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SqliteDbCfg {
    pub path: PathBuf,
}

impl SqliteDbCfg {
    pub fn convert_to_abs_path(&mut self, basepath: &Path) -> anyhow::Result<()> {
        self.path = base::resolve_relative_path(basepath, &self.path)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, strum::EnumString, Copy)]
pub enum DbType {
    #[serde(rename = "mysql")]
    #[strum(serialize = "mysql")]
    MySql,
    #[serde(rename = "sqlite")]
    #[strum(serialize = "sqlite")]
    Sqlite,
}

pub enum DbCfg {
    Mysql(MysqlDbCfg),
    Sqlite(SqliteDbCfg),
}

impl Default for DbType {
    fn default() -> Self {
        Self::MySql
    }
}

pub static DB_TYPE: OnceLock<DbType> = OnceLock::new();

pub static MYSQL_TYPE: static_keys::StaticFalseKey = static_keys::new_static_false_key();
pub static SQLITE_TYPE: static_keys::StaticFalseKey = static_keys::new_static_false_key();

/// 初始化数据库层
pub fn init_db_system(db_type: DbType) {
    tracing::info!("Init db system");
    DB_TYPE.get_or_init(|| {
        tracing::info!("db type: {:?}", db_type);
        match db_type {
            DbType::MySql => unsafe { MYSQL_TYPE.enable() },
            DbType::Sqlite => unsafe { SQLITE_TYPE.enable() },
        };
        db_type.clone()
    });
}

pub fn get_db_type() -> DbType {
    match DB_TYPE.get() {
        Some(db_type) => db_type.clone(),
        None => {
            tracing::error!("Db system has not been inited");
            panic!("Db system has not been inited");
        }
    }
}

/// 根据配置文件生成连接数据库的url
pub fn get_db_url(cfg: &DbCfg) -> anyhow::Result<String> {
    let db_type = get_db_type();
    match db_type {
        DbType::MySql => {
            let cfg = match cfg {
                DbCfg::Mysql(cfg) => cfg,
                DbCfg::Sqlite(_) => {
                    tracing::error!("sqlite database config for mysql database");
                    anyhow::bail!("sqlite database config for mysql database");
                }
            };
            let path = format!(
                "mysql://{}:{}@{}:{}/{}",
                cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
            );
            Ok(path)
        }
        DbType::Sqlite => {
            let cfg = match cfg {
                DbCfg::Sqlite(cfg) => cfg,
                DbCfg::Mysql(_) => {
                    tracing::error!("mysql database config for sqlite database");
                    anyhow::bail!("mysql database config for sqlite database")
                }
            };
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
                tracing::warn!(
                    "Failed to create mysql database: {}.Maybe the database already exists",
                    e
                );
            }
        }
    }
    Ok(())
}

/// connect to database according to url
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

/// Init Database and Running Migrations
pub async fn init_db(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    static INIT: Once = Once::new();
    let db_type = get_db_type();
    let mut flag = match db_type {
        DbType::MySql => false,
        DbType::Sqlite => true,
    };
    INIT.call_once(|| {
        flag = true;
    });
    if flag {
        migration::Migrator::up(db, None).await?;
        tracing::info!("Ran all migrations of databases");
        tracing::info!("Initialized database");
    }
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
