//! new session response

use crate::consts::{self, ID};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Success = 0,
    Failed = 1,
    UpToLimit = 2,
}

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
