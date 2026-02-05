use crate::setting::Setting;
use migration::MigratorTrait;
use sea_orm::{ConnectOptions, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PostgresDbCfg {
    #[serde(default)]
    pub inherit: Option<String>,
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

impl Setting for PostgresDbCfg {}

async fn try_create_postgres_db(
    url: &str,
    run_migration: bool,
) -> anyhow::Result<DatabaseConnection> {
    use sqlx::{Postgres, migrate::MigrateDatabase};
    if !Postgres::database_exists(url).await.unwrap_or(false) {
        tracing::info!("Creating postgres database");
        match Postgres::create_database(url).await {
            Ok(_) => {
                tracing::info!("Created postgres database {}", url);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to create postgres database: {}.Maybe the database already exists",
                    e
                );
            }
        }
    }
    let mut opt = ConnectOptions::new(url);
    opt.connect_timeout(Duration::from_mins(1));
    let db = sea_orm::Database::connect(opt).await?;
    tracing::info!("Ran all migrations of databases");
    if run_migration {
        migration::Migrator::up(&db, None).await?;
    }
    Ok(db)
}

/// connect to the database according to url
pub async fn connect_to_db(url: &str, run_migration: bool) -> anyhow::Result<DatabaseConnection> {
    tracing::info!("Connecting to {}", url);
    try_create_postgres_db(url, run_migration).await
}

/// Init Postgresql Database
pub async fn init_postgres(_db: &DatabaseConnection) -> anyhow::Result<()> {
    tracing::info!("Initialized database");
    Ok(())
}
