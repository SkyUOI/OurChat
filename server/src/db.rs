//! Database

pub mod compatibility;
pub mod file_storage;

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub use compatibility::*;
use config::File;
use migration::MigratorTrait;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MysqlDbCfg {
    pub host: String,
    pub user: String,
    pub db: String,
    pub port: usize,
    pub passwd: String,
}

pub trait DbCfgTrait {
    fn url(&self) -> String;
}

impl DbCfgTrait for MysqlDbCfg {
    fn url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.passwd, self.host, self.port, self.db
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SqliteDbCfg {
    pub path: PathBuf,
}

impl SqliteDbCfg {
    pub fn convert_to_abs_path(&mut self, basepath: &Path) -> anyhow::Result<()> {
        self.path = base::resolve_relative_path(basepath, &self.path)?;
        Ok(())
    }
}

impl DbCfgTrait for SqliteDbCfg {
    fn url(&self) -> String {
        format!("sqlite://{}", self.path.display())
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

#[derive(Debug, Clone)]
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

/// Initialize the database layer
pub fn init_db_system(db_type: DbType) {
    tracing::info!("Init db system");
    DB_TYPE.get_or_init(|| {
        tracing::info!("db type: {:?}", db_type);
        match db_type {
            DbType::MySql => unsafe { MYSQL_TYPE.enable() },
            DbType::Sqlite => unsafe { SQLITE_TYPE.enable() },
        };
        db_type
    });
}

pub fn get_db_type() -> DbType {
    match DB_TYPE.get() {
        Some(db_type) => *db_type,
        None => {
            tracing::error!("Db system has not been inited");
            panic!("Db system has not been inited");
        }
    }
}

/// Generate the url for connecting to the database according to the configuration file
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
            Ok(cfg.url())
        }
        DbType::Sqlite => {
            let cfg = match cfg {
                DbCfg::Sqlite(cfg) => cfg,
                DbCfg::Mysql(_) => {
                    tracing::error!("mysql database config for sqlite database");
                    anyhow::bail!("mysql database config for sqlite database")
                }
            };
            Ok(cfg.url())
        }
    }
}

pub async fn try_create_sqlite_db(url: &str) -> anyhow::Result<DatabaseConnection> {
    use sqlx::{Sqlite, migrate::MigrateDatabase};
    let mut should_run_migrations = false;
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        tracing::info!("Creating sqlite database {}", url);
        match Sqlite::create_database(url).await {
            Ok(_) => {
                tracing::info!("Created sqlite database {}", url);
                should_run_migrations = true;
            }
            Err(e) => {
                tracing::error!("Failed to create sqlite database: {}", e);
                anyhow::bail!("Failed to create sqlite database: {}", e);
            }
        }
    }
    let db = sea_orm::Database::connect(url).await?;
    if should_run_migrations {
        migration::Migrator::up(&db, None).await?;
        tracing::info!("Ran all migrations of databases");
    }
    Ok(db)
}

pub async fn try_create_mysql_db(url: &str) -> anyhow::Result<DatabaseConnection> {
    use sqlx::{MySql, migrate::MigrateDatabase};
    let mut should_run_migrations = false;
    if !MySql::database_exists(url).await.unwrap_or(false) {
        tracing::info!("Creating mysql database");
        match MySql::create_database(url).await {
            Ok(_) => {
                tracing::info!("Created mysql database {}", url);
                should_run_migrations = true;
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to create mysql database: {}.Maybe the database already exists",
                    e
                );
            }
        }
    }
    let db = sea_orm::Database::connect(url).await?;
    if should_run_migrations {
        migration::Migrator::up(&db, None).await?;
        tracing::info!("Ran all migrations of databases");
    }
    Ok(db)
}

/// connect to database according to url
pub async fn connect_to_db(url: &str) -> anyhow::Result<DatabaseConnection> {
    let db_type = get_db_type();
    tracing::info!("Connecting to {}", url);
    Ok(match db_type {
        DbType::MySql => try_create_mysql_db(url).await?,
        DbType::Sqlite => try_create_sqlite_db(url).await?,
    })
}

/// Init Database
pub async fn init_db(_db: &DatabaseConnection) -> anyhow::Result<()> {
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

/// Generate the url for connecting to redis according to the configuration file
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

/// connect to redis database according to url
pub async fn connect_to_redis(url: &str) -> anyhow::Result<deadpool_redis::Pool> {
    let cfg = deadpool_redis::Config::from_url(url);
    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    Ok(pool)
}
