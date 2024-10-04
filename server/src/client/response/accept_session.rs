//! Accept session response

use crate::{client::requests::Status, consts::MessageType};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptSessionResponse {
    pub code: MessageType,
    pub status: Status,
}

impl AcceptSessionResponse {
    pub fn success() -> Self {
        Self {
            code: MessageType::AcceptSessionRes,
            status: Status::Success,
        }
    }

    pub fn failed() -> Self {
        Self {
            code: MessageType::AcceptSessionRes,
            status: Status::AccountLimitation,
        }
    }
}

impl From<AcceptSessionResponse> for Message {
    fn from(value: AcceptSessionResponse) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
