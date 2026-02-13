use std::sync::Arc;
use std::time::Duration;

use crate::SharedData;
use crate::db::redis::redis_key;
use crate::httpserver::EmailClientType;
use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use base::constants;
use base::database::DbPool;
use base::rabbitmq::http_server::VerifyRecord;
use deadpool_redis::redis::AsyncCommands;
use entities::user;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use tokio::fs::read_to_string;

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
    // check if the token is valid and get the email
    let email = match check_token_exist_and_del_token(&param.token, &pool.redis_pool).await {
        Ok(Some(email)) => email,
        Ok(None) => {
            tracing::warn!("Token not found or expired: {}", param.token);
            return Err(StatusCode::BAD_REQUEST);
        }
        Err(e) => {
            tracing::error!("Error while checking token:{:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Update the user's email_verified field
    if let Err(e) = update_user_email_verified(&email, &pool.db_pool).await {
        tracing::error!("Failed to update user email verification status: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
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
        let verification_link = format!(
            "{}v1/verify/confirm?token={}",
            shared_data.cfg().http_cfg.base_url(),
            data.token
        );

        let text_body = format!(
            "please click \"{}\" to verify your email",
            verification_link
        );

        let html_template_path = shared_data
            .cfg()
            .http_cfg
            .verification_html_template_path
            .clone();

        let html_body = if let Some(html_template_path) = html_template_path {
            match read_to_string(&html_template_path).await {
                Err(e) => {
                    tracing::error!("Failed to read {}: {:?}", html_template_path.display(), e);
                    None
                }
                Ok(template) => Some(template.replace("[verification_link]", &verification_link)),
            }
        } else {
            None
        };

        if let Err(e) = email_client
            .send(
                user_mailbox,
                format!("{} Verification", constants::APP_NAME),
                text_body,
                html_body,
            )
            .await
        {
            Err(e)?
        };
    }
    let expiry = shared_data.cfg().user_setting.verify_email_expiry;
    add_token(&data.token, &data.email, expiry, &db.redis_pool).await?;
    Ok(())
}

fn mapped_to_redis(key: &str) -> String {
    redis_key!("verify:{}", key)
}

pub async fn check_token_exist_and_del_token(
    token: &str,
    conn: &deadpool_redis::Pool,
) -> anyhow::Result<Option<String>> {
    let mut conn = conn.get().await?;
    let key = mapped_to_redis(token);
    let email: Option<String> = conn.get(&key).await?;
    let _: () = conn.del(&key).await?;
    Ok(email)
}

async fn add_token(
    token: &str,
    email: &str,
    ex_time: Duration,
    conn: &deadpool_redis::Pool,
) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    let _: () = conn
        .set_ex(mapped_to_redis(token), email, ex_time.as_secs())
        .await?;
    Ok(())
}

async fn update_user_email_verified(
    email: &str,
    db_pool: &sea_orm::DatabaseConnection,
) -> anyhow::Result<()> {
    // Find the user by email
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db_pool)
        .await?;

    if let Some(user) = user {
        // Update the email_verified field to true
        let mut user_active: user::ActiveModel = user.into();
        user_active.email_verified = ActiveValue::Set(true);
        user_active.update(db_pool).await?;
        tracing::info!("Successfully verified email for user: {}", email);
    } else {
        tracing::warn!("User not found for email: {}", email);
    }

    Ok(())
}

pub fn config() -> axum::Router<DbPool> {
    axum::Router::new().route("/verify/confirm", get(verify_token))
}
