use crate::{
    DbPool, SharedData,
    component::{EmailClient, EmailSender, MockEmailSender},
    consts,
};
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use anyhow::Context;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Serialize, Deserialize)]
struct VerifyForm {
    email: String,
}
#[derive(Debug, Deserialize)]
struct Param {
    token: String,
}

#[get("/verify/confirm")]
async fn verify_token(
    req: HttpRequest,
    param: web::Query<Param>,
) -> Result<impl Responder, actix_web::Error> {
    let conn = match req.app_data::<DbPool>() {
        None => {
            tracing::error!("No redis connection");
            return Ok(HttpResponse::InternalServerError());
        }
        Some(conn) => &conn.redis_pool,
    };
    // check if token is valid
    let ret = if match check_token(&param.token, conn).await {
        Ok(data) => data,
        Err(_) => {
            tracing::error!("check token error");
            return Ok(HttpResponse::InternalServerError());
        }
    } {
        match req.app_data::<Arc<crate::SharedData<EmailClient>>>() {
            Some(data) => {
                if let Some(d) = data.verify_record.remove(&param.token) {
                    d.1.notify_waiters();
                }
            }
            None => match req.app_data::<Arc<crate::SharedData<MockEmailSender>>>() {
                None => {
                    tracing::error!("No shared data");
                    return Ok(HttpResponse::InternalServerError());
                }
                Some(data) => {
                    if let Some(d) = data.verify_record.remove(&param.token) {
                        d.1.notify_waiters();
                    }
                }
            },
        }
        HttpResponse::Ok()
    } else {
        HttpResponse::BadRequest()
    };
    Ok(ret)
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VerifyRecord {
    token: String,
    email: String,
}

impl VerifyRecord {
    pub fn new(email: String, token: String) -> Self {
        Self { token, email }
    }
}

pub async fn verify_client(
    db: &DbPool,
    shared_data: Arc<crate::SharedData<impl EmailSender>>,
    data: VerifyRecord,
    notify: Arc<Notify>,
) -> anyhow::Result<()> {
    let cfg = &shared_data.cfg.main_cfg;
    if let Some(ref email_client) = shared_data.email_client {
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
                    "please click \"http://{}:{}/v1/verify/confirm?token={}\" to verify your email",
                    cfg.ip, cfg.http_port, data.token
                ),
            )
            .await
        {
            Err(e)?
        };
    }
    add_token(&data.token, &db.redis_pool).await?;
    shared_data.verify_record.insert(data.token, notify);
    Ok(())
}

fn mapped_to_redis(key: &str) -> String {
    format!("verify:{}", key)
}

async fn check_token(token: &str, conn: &deadpool_redis::Pool) -> anyhow::Result<bool> {
    let mut conn = conn.get().await?;
    let ret: bool = conn.exists(mapped_to_redis(token)).await?;
    let _: () = conn.del(token).await?;
    Ok(ret)
}

async fn clean_useless_notifier<T: EmailSender>(
    map: &Arc<SharedData<T>>,
    conn: &deadpool_redis::Pool,
) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    for i in &map.verify_record {
        let ret: bool = conn.exists(mapped_to_redis(i.key())).await?;
        if !ret {
            map.verify_record.remove(i.key());
        }
    }
    Ok(())
}

pub async fn regularly_clean_notifier<T: EmailSender>(
    map: Arc<SharedData<T>>,
    conn: deadpool_redis::Pool,
) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        if let Err(e) = clean_useless_notifier(&map, &conn).await {
            tracing::error!("unable to clean notifier:{e}");
        }
    }
}

async fn add_token(token: &str, conn: &deadpool_redis::Pool) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    let _: () = conn
        .set_ex(
            mapped_to_redis(token),
            1,
            std::time::Duration::from_mins(5).as_secs(),
        )
        .await?;
    Ok(())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_token);
}
