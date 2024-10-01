use actix_web::{HttpResponse, Responder, get, web};
use dashmap::DashMap;
use lettre::AsyncTransport;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tokio::{
    sync::{mpsc, oneshot},
    time::Instant,
};

#[derive(Serialize, Deserialize)]
struct VerifyForm {
    email: String,
}

struct _EmailSender {}

#[cfg(test)]
#[mockall::automock]
impl _EmailSender {
    fn _new(_addr: String) -> Self {
        _EmailSender {}
    }
}

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
        shared_data: Arc<crate::SharedData>,
        mut request_receiver: mpsc::Receiver<VerifyRequest>,
    ) {
        let cfg = &shared_data.cfg.main_cfg;
        while let Some((data, resp_sender)) = request_receiver.recv().await {
            if shared_data.shared_state.email_available {
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
                        cfg.ip, cfg.http_port, data.token
                    )) {
                    Err(e) => {
                        resp_sender.send(Err(anyhow::anyhow!(e))).unwrap();
                        continue;
                    }
                    Ok(email) => email,
                };
                match shared_data
                    .shared_state
                    .email_client
                    .as_ref()
                    .unwrap()
                    .send(email)
                    .await
                {
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
    #[tokio::test]
    async fn test_email_send() {}
}
