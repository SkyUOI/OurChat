//! Database

pub mod file_storage;
pub mod helper;
pub mod messages;
pub mod session;
pub mod user;

use config::File;
use migration::MigratorTrait;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresDbCfg {
    pub host: String,
    pub user: String,
    pub db: String,
    pub port: usize,
    pub passwd: String,
}

impl PostgresDbCfg {
    pub fn url(&self) -> String {
        if self.passwd.is_empty() {
            format!(
                "postgres://{}@{}:{}/{}",
                self.user, self.host, self.port, self.db
            )
        } else {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.user, self.passwd, self.host, self.port, self.db
            )
        }
    }
}

/// Initialize the database layer
pub fn init_db_system() {
    tracing::info!("Init db system");
}

async fn try_create_postgres_db(url: &str) -> anyhow::Result<DatabaseConnection> {
    use sqlx::{Postgres, migrate::MigrateDatabase};
    let mut should_run_migrations = false;
    if !Postgres::database_exists(url).await.unwrap_or(false) {
        tracing::info!("Creating postgres database");
        match Postgres::create_database(url).await {
            Ok(_) => {
                tracing::info!("Created postgres database {}", url);
                should_run_migrations = true;
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to create postgres database: {}.Maybe the database already exists",
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
    tracing::info!("Connecting to {}", url);
    try_create_postgres_db(url).await
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
