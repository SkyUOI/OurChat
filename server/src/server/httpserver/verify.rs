use actix_web::{HttpResponse, Responder, get, web};
use dashmap::DashMap;
use lettre::AsyncTransport;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tokio::{
    sync::{mpsc, oneshot},
    time::Instant,
};

use crate::component::EmailSender;

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
    manager: web::Data<VerifyManager>,
    param: web::Query<Param>,
) -> impl Responder {
    // check if token is valid
    match manager.get_records(&param.token) {
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
        shared_data: Arc<crate::SharedData<impl EmailSender>>,
        mut request_receiver: mpsc::Receiver<VerifyRequest>,
    ) {
        let cfg = &shared_data.cfg.main_cfg;
        while let Some((data, resp_sender)) = request_receiver.recv().await {
            if let Some(ref email_client) = shared_data.email_client {
                if let Err(e) = email_client
                    .send(
                        format!("User <{}>", data.email).parse().unwrap(),
                        "OurChat Verification",
                        format!(
                            "please click \"{}:{}/verify/{}\" to verify your email",
                            cfg.ip, cfg.http_port, data.token
                        ),
                    )
                    .await
                {
                    resp_sender.send(Err(e)).unwrap();
                    continue;
                };
            }
            manager.records.insert(data.token.clone(), Token {
                record: data,
                time: Instant::now(),
            });
            if let Err(e) = resp_sender.send(Ok(())) {
                tracing::error!("send response error,{:?}", e);
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
    #[tokio::test]
    async fn test_email_send() {}
}
