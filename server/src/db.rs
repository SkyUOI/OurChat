use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct DbCfg {
    host: String,
    user: String,
    db: String,
    port: usize,
    passwd: String,
}

pub async fn connect_to_db(path: &str) -> anyhow::Result<sqlx::MySqlPool> {
    let json = std::fs::read_to_string(path)?;
    let cfg: DbCfg = serde_json::from_str(&json)?;
    let path = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
    );
    log::info!("Connecting to {}", path);
    Ok(sqlx::MySqlPool::connect(&path).await?)
}

pub async fn init_db(db: &sqlx::MySqlPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user(
            id BIGINT UNSIGNED,
            ocid CHAR(10),
            passwd CHAR(64),
            name CHAR(15),
            email CHAR(120),
            date INT,
            PRIMARY KEY(id),
            UNIQUE KEY(ocid),
            UNIQUE KEY(email)
            )DEFAULT CHARSET=utf8mb4;"#,
    )
    .execute(db)
    .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS friend(
            user_id BIGINT UNSIGNED,
            friend_id BIGINT UNSIGNED,
            name CHAR(15),
            PRIMARY KEY(user_id)
            )DEFAULT CHARSET=utf8mb4;"#,
    )
    .execute(db)
    .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS chat(
            group_id BIGINT UNSIGNED,
            user_id BIGINT UNSIGNED,
            name CHAR(15),
            group_name CHAR(30),
            PRIMARY KEY(group_id)
            )DEFAULT CHARSET=utf8mb4;"#,
    )
    .execute(db)
    .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS chatgroup(
            group_id BIGINT UNSIGNED,
            group_name CHAR(30),
            PRIMARY KEY(group_id)
            )DEFAULT CHARSET=utf8mb4;"#,
    )
    .execute(db)
    .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user_chat_msg(
            user_id BIGINT UNSIGNED NOT NULL,
            chat_msg_id INT UNSIGNED NOT NULL
            )DEFAULT CHARSET=utf8mb4;"#,
    )
    .execute(db)
    .await?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user_chat_id(
            chat_msg_id INT UNSIGNED AUTO_INCREMENT,
            msg_type INT,
            msg_data VARCHAR(8000),
            sender_id BIGINT UNSIGNED,
            PRIMARY KEY(chat_msg_id)
            )DEFAULT CHARSET=utf8mb4;"#,
    )
    .execute(db)
    .await?;
    log::info!("Initialized database");
    Ok(())
}
