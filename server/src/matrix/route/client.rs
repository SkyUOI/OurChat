use axum::{Json, routing::get};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResponse {
    pub unstable_features: HashMap<String, bool>,
    pub versions: Vec<String>,
}

pub async fn versions() -> Json<VersionResponse> {
    let response = VersionResponse {
        unstable_features: collection_literals::collection! {},
        versions: vec!["v1.13".to_owned()],
    };
    Json(response)
}

pub fn configure_route() -> axum::Router {
    axum::Router::new().route("/versions", get(versions))
}
