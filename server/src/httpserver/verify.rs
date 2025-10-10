use std::sync::Arc;
use std::time::Duration;

use crate::SharedData;
use crate::httpserver::EmailClientType;
use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use base::consts;
use base::database::DbPool;
use base::rabbitmq::http_server::VerifyRecord;
use deadpool_redis::redis::AsyncCommands;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Param {
    token: String,
}

#[tracing::instrument]
#[axum::debug_handler]
async fn verify_token(
    State(pool): State<DbPool>,
    Query(param): Query<Param>,
) -> Result<(), StatusCode> {
    // check if the token is valid
    if match check_token_exist_and_del_token(&param.token, &pool.redis_pool).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Error while checking token:{:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    } {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn verify_client(
    db: &DbPool,
    email_client: &Option<EmailClientType>,
    data: VerifyRecord,
    shared_data: &Arc<SharedData>,
) -> anyhow::Result<()> {
    if let Some(email_client) = email_client {
        let user_mailbox = format!("User <{}>", data.email);
        let user_mailbox = match user_mailbox
            .parse()
            .with_context(|| format!("email {user_mailbox} parse failed"))
        {
            Ok(mailbox) => mailbox,
            Err(e) => Err(e)?,
        };

        let text_body = format!(
            "please click \"{}v1/verify/confirm?token={}\" to verify your email",
            shared_data.cfg.http_cfg.base_url(),
            data.token
        );

        let html_template = std::fs::read_to_string("templates/email.html")
            .with_context(|| "Failed to read email.html template")?;
        
        let html_body = Some(html_template.replace(
            "[verification_link]",
            &format!(
                "{}v1/verify/confirm?token={}",
                shared_data.cfg.http_cfg.base_url(),
                data.token
            ),
        ));

        if let Err(e) = email_client
            .send(
                user_mailbox,
                format!("{} Verification", consts::APP_NAME),
                text_body,
                html_body,
            )
            .await
        {
            Err(e)?
        };
    }
    add_token(
        &data.token,
        shared_data.cfg.user_setting.verify_email_expiry,
        &db.redis_pool,
    )
    .await?;
    Ok(())
}

fn mapped_to_redis(key: &str) -> String {
    format!("verify:{key}")
}

pub async fn check_token_exist_and_del_token(
    token: &str,
    conn: &deadpool_redis::Pool,
) -> anyhow::Result<bool> {
    let mut conn = conn.get().await?;
    let ret: bool = conn.exists(mapped_to_redis(token)).await?;
    let _: () = conn.del(token).await?;
    Ok(ret)
}

async fn add_token(
    token: &str,
    ex_time: Duration,
    conn: &deadpool_redis::Pool,
) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    let _: () = conn
        .set_ex(mapped_to_redis(token), 1, ex_time.as_secs())
        .await?;
    Ok(())
}

pub fn config() -> axum::Router<DbPool> {
    axum::Router::new().route("/verify/confirm", get(verify_token))
}
