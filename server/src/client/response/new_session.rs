//! new session response

use crate::client::requests::Status;
use crate::consts::{self, ID};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSessionResponse {
    pub code: consts::MessageType,
    pub status: Status,
    pub session_id: Option<ID>,
}

impl NewSessionResponse {
    pub fn success(session_id: ID) -> Self {
        Self {
            code: consts::MessageType::NewSessionRes,
            status: Status::Success,
            session_id: Some(session_id),
        }
    }

    pub fn failed(status: Status) -> Self {
        Self {
            code: consts::MessageType::NewSessionRes,
            status,
            session_id: None,
        }
    }
}

impl From<NewSessionResponse> for Message {
    fn from(value: NewSessionResponse) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
