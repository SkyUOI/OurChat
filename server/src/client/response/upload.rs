//! Upload Files

use crate::{client::requests::Status, consts};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub code: consts::MessageType,
    status: Status,
    key: Option<String>,
    hash: Option<String>,
}

impl UploadResponse {
    pub fn success(key: String, hash: String) -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            key: Some(key),
            hash: Some(hash),
            status: Status::Success,
        }
    }

    pub fn limited() -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            key: None,
            hash: None,
            status: Status::AccountLimitation,
        }
    }
}

impl From<UploadResponse> for Message {
    fn from(value: UploadResponse) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
