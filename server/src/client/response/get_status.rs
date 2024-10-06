//! Response of unregistering

use crate::{client::requests, consts};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub code: consts::MessageType,
    pub status: requests::Status,
}

impl GetStatusResponse {
    pub fn normal() -> Self {
        Self {
            code: consts::MessageType::GetStatus,
            status: requests::Status::Success,
        }
    }

    pub fn maintaining() -> Self {
        Self {
            code: consts::MessageType::GetStatus,
            status: requests::Status::Maintaining,
        }
    }
}

impl From<GetStatusResponse> for Message {
    fn from(value: GetStatusResponse) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
