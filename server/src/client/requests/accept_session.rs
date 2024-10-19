//! Accept session request

use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptSessionRequest {
    pub code: MessageType,
    pub session_id: String,
}

impl AcceptSessionRequest {
    pub fn new(session_id: String) -> Self {
        Self {
            code: MessageType::AcceptSession,
            session_id,
        }
    }
}
