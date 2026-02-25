use crate::setting::Setting;
use migration::MigratorTrait;
use sea_orm::{DatabaseConnection, SqlxPostgresPoolConnection};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

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

impl Setting for PostgresDbCfg {}

impl crate::setting::PathConvert for PostgresDbCfg {
    fn convert_to_abs_path(&mut self, _full_basepath: &std::path::Path) -> anyhow::Result<()> {
        Ok(())
    }
}

async fn try_create_postgres_db(
    url: &str,
    run_migration: bool,
) -> anyhow::Result<(DatabaseConnection, PgPool)> {
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

    // Create SQLx connection pool for monitoring
    let pool = PgPoolOptions::new().connect(url).await?;
    let db = DatabaseConnection::SqlxPostgresPoolConnection(SqlxPostgresPoolConnection::from(
        pool.clone(),
    ));

    tracing::info!("Ran all migrations of databases");
    if run_migration {
        migration::Migrator::up(&db, None).await?;
    }

    Ok((db, pool))
}

/// connect to the database according to url
pub async fn connect_to_db(
    url: &str,
    run_migration: bool,
) -> anyhow::Result<(DatabaseConnection, PgPool)> {
    tracing::info!("Connecting to {}", url);
    try_create_postgres_db(url, run_migration).await
}

/// Init Postgresql Database
pub async fn init_postgres(_db: &DatabaseConnection) -> anyhow::Result<()> {
    tracing::info!("Initialized database");
    Ok(())
}
