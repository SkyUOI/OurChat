pub mod user;

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

pub fn get_db_url(path: &str) -> anyhow::Result<String> {
    let json = std::fs::read_to_string(path)?;
    let cfg: DbCfg = serde_json::from_str(&json)?;
    let path = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
    );
    Ok(path)
}

pub async fn connect_to_db(url: &str) -> anyhow::Result<sea_orm::DatabaseConnection> {
    log::info!("Connecting to {}", url);
    Ok(sea_orm::Database::connect(url).await?)
}

pub async fn init_db(db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"CREATE TABLE IF NOT EXISTS user(
            id BIGINT UNSIGNED,
            ocid CHAR(10) NOT NULL,
            passwd CHAR(64) NOT NULL,
            name CHAR(15) NOT NULL,
            email CHAR(120) NOT NULL,
            time BIGINT UNSIGNED NOT NULL,
            PRIMARY KEY(id),
            UNIQUE KEY(ocid),
            UNIQUE KEY(email)
            )DEFAULT CHARSET=utf8mb4;"#
            .to_string(),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"CREATE TABLE IF NOT EXISTS friend(
            user_id BIGINT UNSIGNED,
            friend_id BIGINT UNSIGNED NOT NULL,
            name CHAR(15) NOT NULL,
            PRIMARY KEY(user_id)
            )DEFAULT CHARSET=utf8mb4;"#
            .to_string(),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"CREATE TABLE IF NOT EXISTS chat(
            group_id BIGINT UNSIGNED,
            user_id BIGINT UNSIGNED NOT NULL,
            name CHAR(15) NOT NULL,
            group_name CHAR(30) NOT NULL,
            PRIMARY KEY(group_id)
            )DEFAULT CHARSET=utf8mb4;"#
            .to_string(),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"CREATE TABLE IF NOT EXISTS chatgroup(
            group_id BIGINT UNSIGNED,
            group_name CHAR(30) NOT NULL,
            PRIMARY KEY(group_id)
            )DEFAULT CHARSET=utf8mb4;"#
            .to_string(),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"CREATE TABLE IF NOT EXISTS user_chat_msg(
            user_id BIGINT UNSIGNED NOT NULL,
            chat_msg_id INT UNSIGNED NOT NULL,
            PRIMARY KEY(chat_msg_id)
            )DEFAULT CHARSET=utf8mb4;"#
            .to_string(),
    ))
    .await?;
    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"CREATE TABLE IF NOT EXISTS user_chat_id(
            chat_msg_id INT UNSIGNED AUTO_INCREMENT,
            msg_type INT UNSIGNED NOT NULL,
            msg_data VARCHAR(8000) NOT NULL,
            sender_id BIGINT UNSIGNED NOT NULL,
            PRIMARY KEY(chat_msg_id)
            )DEFAULT CHARSET=utf8mb4;"#
            .to_string(),
    ))
    .await?;
    log::info!("Initialized database");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct RedisCfg {
    host: String,
    port: usize,
    passwd: String,
    user: String,
}

pub fn get_redis_url(path: &str) -> anyhow::Result<String> {
    let json = std::fs::read_to_string(path)?;
    let cfg: RedisCfg = serde_json::from_str(&json)?;
    let path = format!(
        "redis://{}:{}@{}:{}/",
        cfg.user, cfg.passwd, cfg.host, cfg.port
    );
    Ok(path)
}

pub async fn connect_to_redis(url: &str) -> anyhow::Result<redis::Client> {
    let client = redis::Client::open(url)?;
    Ok(client)
}
