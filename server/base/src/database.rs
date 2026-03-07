use crate::database::postgres::PostgresDbCfg;
use crate::database::redis_cfg::RedisCfg;
use sea_orm::DatabaseConnection;
use sqlx::PgPool;

pub mod postgres;
pub mod redis_cfg;

/// The database connection pool, redis connection manager
/// you can clone it freely without many extra cost
#[derive(Debug, Clone)]
pub struct DbPool {
    pub db_pool: DatabaseConnection,
    pub pg_pool: PgPool,
    pub redis_conn: redis::aio::ConnectionManager,
}

impl DbPool {
    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.db_pool.clone().close().await?;
        Ok(())
    }

    pub async fn build(
        postgres: &PostgresDbCfg,
        redis: &RedisCfg,
        run_migration: bool,
    ) -> anyhow::Result<Self> {
        let (db_pool, pg_pool) = postgres::connect_to_db(&postgres.url(), run_migration).await?;
        let redis_conn = redis_cfg::connect_to_redis(&redis.get_redis_url()?).await?;
        Ok(Self {
            db_pool,
            pg_pool,
            redis_conn,
        })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        postgres::init_postgres(&self.db_pool).await?;
        Ok(())
    }

    /// Get a clone of the redis connection manager.
    /// The ConnectionManager is cheap to clone and can be used directly.
    pub fn redis(&self) -> redis::aio::ConnectionManager {
        self.redis_conn.clone()
    }
}
