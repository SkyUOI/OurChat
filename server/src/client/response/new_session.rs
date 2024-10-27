//! new session response

use crate::consts::{self, ID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSessionResponse {
    pub code: consts::MessageType,
    pub session_id: ID,
}

impl NewSessionResponse {
    pub fn success(session_id: ID) -> Self {
        Self {
            code: consts::MessageType::NewSessionRes,
            session_id,
        }
    }
}
