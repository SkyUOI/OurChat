use actix_web::{HttpResponse, Responder, get, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResponse {
    pub unstable_features: HashMap<String, bool>,
    pub versions: Vec<String>,
}
#[get("/versions")]
pub async fn versions() -> impl Responder {
    let response = VersionResponse {
        unstable_features: collection_literals::collection! {},
        versions: vec!["v1.13".to_owned()],
    };
    HttpResponse::Ok().json(response)
}

pub fn configure_client(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/client").service(versions);
    cfg.service(scope);
}
