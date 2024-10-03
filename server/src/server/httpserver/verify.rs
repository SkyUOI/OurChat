use crate::component::EmailSender;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use anyhow::Context;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

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
    let conn = match req.app_data::<deadpool_redis::Pool>() {
        None => {
            tracing::error!("No redis connection");
            return Ok(HttpResponse::InternalServerError());
        }
        Some(conn) => conn,
    };
    // check if token is valid
    let ret = if match check_token(&param.token, conn).await {
        Ok(data) => data,
        Err(_) => {
            tracing::error!("check token error");
            return Ok(HttpResponse::InternalServerError());
        }
    } {
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

pub type VerifyRequest = (VerifyRecord, oneshot::Sender<anyhow::Result<()>>);

pub async fn add_record(
    redis: deadpool_redis::Pool,
    shared_data: Arc<crate::SharedData<impl EmailSender>>,
    mut request_receiver: mpsc::Receiver<VerifyRequest>,
) {
    let cfg = &shared_data.cfg.main_cfg;
    while let Some((data, resp_sender)) = request_receiver.recv().await {
        if let Some(ref email_client) = shared_data.email_client {
            let user_mailbox = format!("User <{}>", data.email);
            let user_mailbox = match user_mailbox
                .parse()
                .with_context(|| format!("email {} parse failed", user_mailbox))
            {
                Ok(mailbox) => mailbox,
                Err(e) => {
                    resp_sender.send(Err(e)).unwrap();
                    continue;
                }
            };

            if let Err(e) = email_client
                    .send(
                        user_mailbox,
                        "OurChat Verification",
                        format!(
                            "please click \"http://{}:{}/v1/verify/confirm?token={}\" to verify your email",
                            cfg.ip, cfg.http_port, data.token
                        ),
                    )
                    .await
                {
                    resp_sender.send(Err(e)).unwrap();
                    continue;
                };
        }
        if let Err(e) = add_token(&data.token, &redis).await {
            tracing::error!("add token error,{:?}", e);
            continue;
        }
        if let Err(e) = resp_sender.send(Ok(())) {
            tracing::error!("send response error,{:?}", e);
        };
    }
}

async fn wrap_check_token(token: &str, conn: &deadpool_redis::Pool) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    let _: () = conn.del(token).await?;
    Ok(())
}
async fn check_token(token: &str, conn: &deadpool_redis::Pool) -> anyhow::Result<bool> {
    let ret = wrap_check_token(token, conn).await.is_ok();
    Ok(ret)
}

async fn add_token(token: &str, conn: &deadpool_redis::Pool) -> anyhow::Result<()> {
    let mut conn = conn.get().await?;
    let _: () = conn
        .set_ex(token, 1, std::time::Duration::from_mins(5).as_secs())
        .await?;
    Ok(())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_token);
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_email_send() {}
}
