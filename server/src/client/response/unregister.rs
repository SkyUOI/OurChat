//! Unregister Response

use crate::{client::requests, consts};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub code: consts::MessageType,
    pub status: requests::Status,
}

impl UnregisterResponse {
    pub fn new(status: requests::Status) -> Self {
        Self {
            code: consts::MessageType::UnregisterRes,
            status,
        }
    }
}

impl From<UnregisterResponse> for Message {
    fn from(value: UnregisterResponse) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
