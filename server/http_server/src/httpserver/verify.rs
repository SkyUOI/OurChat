use crate::{Cfg, EmailClientType, MainCfg};
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use anyhow::Context;
use base::consts;
use base::consts::VERIFY_EMAIL_EXPIRE;
use base::database::DbPool;
use base::rabbitmq::http_server::VerifyRecord;
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct VerifyForm {
    email: String,
}
#[derive(Debug, Deserialize)]
struct Param {
    token: String,
}

#[get("/verify/confirm")]
#[tracing::instrument]
async fn verify_token(
    req: HttpRequest,
    param: web::Query<Param>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = match req.app_data::<DbPool>() {
        None => {
            tracing::error!("No Database connection");
            return Ok(HttpResponse::InternalServerError());
        }
        Some(pool) => pool,
    };
    // check if the token is valid
    let ret = if match check_token_exist_and_del_token(&param.token, &pool.redis_pool).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Error while checking token:{:?}", e);
            return Ok(HttpResponse::InternalServerError());
        }
    } {
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    };
    Ok(ret)
}

pub async fn verify_client(
    db: &DbPool,
    email_client: &Option<EmailClientType>,
    data: VerifyRecord,
    cfg: &web::Data<Cfg>,
) -> anyhow::Result<()> {
    if let Some(email_client) = email_client {
        let user_mailbox = format!("User <{}>", data.email);
        let user_mailbox = match user_mailbox
            .parse()
            .with_context(|| format!("email {} parse failed", user_mailbox))
        {
            Ok(mailbox) => mailbox,
            Err(e) => Err(e)?,
        };

        if let Err(e) = email_client
            .send(
                user_mailbox,
                format!("{} Verification", consts::APP_NAME),
                format!(
                    "please click \"{}v1/verify/confirm?token={}\" to verify your email",
                    cfg.main_cfg.base_url(),
                    data.token
                ),
            )
            .await
        {
            Err(e)?
        };
    }
    add_token(&data.token, &db.redis_pool).await?;
    Ok(())
}

fn mapped_to_redis(key: &str) -> String {
    format!("verify:{}", key)
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

async fn add_token(token: &str, conn: &deadpool_redis::Pool) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    let _: () = conn
        .set_ex(mapped_to_redis(token), 1, VERIFY_EMAIL_EXPIRE.as_secs())
        .await?;
    Ok(())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_token);
}
