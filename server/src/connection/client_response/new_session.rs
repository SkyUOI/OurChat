//! new session response

use crate::consts::{self, ID};
use crate::requests::Status;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSessionResponse {
    pub code: consts::MessageType,
    pub status: Status,
    pub session_id: Option<ID>,
}

impl NewSessionResponse {
    pub fn success(session_id: ID) -> Self {
        Self {
            code: consts::MessageType::NewSessionResponse,
            status: Status::Success,
            session_id: Some(session_id),
        }
    }

    pub fn failed(status: Status) -> Self {
        Self {
            code: consts::MessageType::NewSessionResponse,
            status,
            session_id: None,
        }
    }
}
