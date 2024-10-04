//! Accept session request

use crate::consts::MessageType;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptSession {
    pub code: MessageType,
    pub session_id: String,
}

impl AcceptSession {
    pub fn new(session_id: String) -> Self {
        Self {
            code: MessageType::AcceptSession,
            session_id,
        }
    }
}

impl From<AcceptSession> for Message {
    fn from(value: AcceptSession) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
