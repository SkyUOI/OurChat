use std::sync::LazyLock;

use actix_web::{get, web, HttpResponse, Responder};
use dashmap::DashSet;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::Instant};

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

#[get("/verify/{token}")]
async fn verify_token() -> impl Responder {
    HttpResponse::Ok()
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

pub struct VerifyManager {
    // TODO:add timeout
    records: DashSet<(VerifyRecord, Instant)>,
}

impl VerifyManager {
    pub fn new() -> Self {
        Self {
            records: DashSet::new(),
        }
    }

    pub async fn add_record(
        manager: web::Data<VerifyManager>,
        mut request_receiver: mpsc::Receiver<VerifyRecord>,
    ) {
        while let Some(data) = request_receiver.recv().await {
            manager.records.insert((data, Instant::now()));
        }
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
