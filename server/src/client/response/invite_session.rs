use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteSession {
    pub code: MessageType,
    pub available_duration: u64,
    pub session_id: String,
    pub inviter_id: String,
    pub voice_message: String,
}

impl InviteSession {
    pub fn new(
        available_duration: u64,
        session_id: String,
        inviter_id: String,
        voice_message: String,
    ) -> Self {
        Self {
            code: MessageType::InviteSession,
            available_duration,
            session_id,
            inviter_id,
            voice_message,
        }
    }
}
