use actix_web::{HttpResponse, Responder, get, web};
use dashmap::DashMap;
use lettre::{AsyncSmtpTransport, AsyncTransport, transport::smtp::authentication::Credentials};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tokio::{
    sync::{mpsc, oneshot},
    time::Instant,
};

use crate::{EMAIL_AVAILABLE, global_cfg};

#[derive(Serialize, Deserialize)]
struct VerifyForm {
    email: String,
}

struct EmailSender {}

#[cfg(test)]
#[mockall::automock]
impl EmailSender {
    fn new(addr: String) -> Self {
        EmailSender {}
    }
}

pub static MAILER: LazyLock<Option<AsyncSmtpTransport<lettre::Tokio1Executor>>> =
    LazyLock::new(|| {
        if !*EMAIL_AVAILABLE {
            return None;
        }
        let cfg = global_cfg(None);
        let creds = Credentials::new(
            cfg.email_address.clone().unwrap(),
            cfg.smtp_password.clone().unwrap(),
        );
        Some(
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&cfg.smtp_address.clone().unwrap())
                .unwrap()
                .credentials(creds)
                .build(),
        )
    });

#[get("/verify/{token}")]
async fn verify_token(
    manager: web::Data<VerifyManager>,
    token: web::Path<String>,
) -> impl Responder {
    // check if token is valid
    match manager.get_records(&token.into_inner()) {
        None => HttpResponse::BadRequest(),
        Some(token) => {
            manager.remove_record(&token.record.token);
            HttpResponse::Ok()
        }
    }
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

#[derive(Debug, PartialEq, Eq, Hash)]
struct Token {
    record: VerifyRecord,
    time: Instant,
}

pub struct VerifyManager {
    // TODO:add timeout
    records: DashMap<String, Token>,
}

pub type VerifyRequest = (VerifyRecord, oneshot::Sender<anyhow::Result<()>>);

impl VerifyManager {
    pub fn new() -> Self {
        Self {
            records: DashMap::new(),
        }
    }

    pub async fn add_record(
        manager: web::Data<VerifyManager>,
        mut request_receiver: mpsc::Receiver<VerifyRequest>,
    ) {
        let cfg = global_cfg(None);
        while let Some((data, resp_sender)) = request_receiver.recv().await {
            if *EMAIL_AVAILABLE {
                let email = match lettre::Message::builder()
                    .from(
                        format!("OurChat <{}>", cfg.email_address.as_ref().unwrap())
                            .parse()
                            .unwrap(),
                    )
                    .to(format!("User <{}>", data.email).parse().unwrap())
                    .subject("OurChat Verification")
                    .body(format!(
                        "please click \"{}:{}/verify/{}\" to verify your email",
                        cfg.ip,
                        cfg.http_port.unwrap(),
                        data.token
                    )) {
                    Err(e) => {
                        resp_sender.send(Err(anyhow::anyhow!(e))).unwrap();
                        continue;
                    }
                    Ok(email) => email,
                };
                match MAILER.as_ref().unwrap().send(email).await {
                    Err(e) => {
                        resp_sender.send(Err(anyhow::anyhow!(e))).unwrap();
                        continue;
                    }
                    Ok(_) => {}
                };
            }
            manager.records.insert(data.token.clone(), Token {
                record: data,
                time: Instant::now(),
            });
            match resp_sender.send(Ok(())) {
                Err(e) => {
                    tracing::error!("send response error,{:?}", e);
                }
                Ok(_) => {}
            };
        }
    }

    fn get_records(&self, name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, Token>> {
        self.records.get(name)
    }

    fn remove_record(&self, name: &str) {
        self.records.remove(name);
    }
}

pub static MANAGER: LazyLock<web::Data<VerifyManager>> =
    LazyLock::new(|| web::Data::new(VerifyManager::new()));

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(verify_token).app_data(MANAGER.clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_send() {}
}
