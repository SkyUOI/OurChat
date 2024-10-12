use crate::consts::{ID, MessageType, SessionID, TimeStamp};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteSession {
    pub code: MessageType,
    pub expire_timestamp: TimeStamp,
    pub session_id: SessionID,
    pub inviter_id: String,
    pub message: String,
}

impl InviteSession {
    pub fn new(
        expire_timestamp: TimeStamp,
        session_id: SessionID,
        inviter_id: String,
        message: String,
    ) -> Self {
        Self {
            code: MessageType::InviteSession,
            expire_timestamp,
            session_id,
            inviter_id,
            message,
        }
    }
}

impl From<InviteSession> for Message {
    fn from(value: InviteSession) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
