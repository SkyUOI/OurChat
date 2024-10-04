use crate::consts::{MessageType, TimeStamp};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteSession {
    pub code: MessageType,
    pub expire_timestamp: TimeStamp,
    pub session_id: String,
    pub inviter_id: String,
    pub voice_message: String,
}

impl InviteSession {
    pub fn new(
        expire_timestamp: TimeStamp,
        session_id: String,
        inviter_id: String,
        voice_message: String,
    ) -> Self {
        Self {
            code: MessageType::InviteSession,
            expire_timestamp,
            session_id,
            inviter_id,
            voice_message,
        }
    }
}

impl From<InviteSession> for Message {
    fn from(value: InviteSession) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
