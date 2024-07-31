//! 注销

use crate::consts;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum Status {
    Success = 0,
    Failed = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub code: consts::MessageType,
    pub status: Status,
}

impl UnregisterResponse {
    pub fn success() -> Self {
        Self {
            code: consts::MessageType::Unregister,
            status: Status::Success,
        }
    }

    pub fn failed() -> Self {
        Self {
            code: consts::MessageType::Unregister,
            status: Status::Failed,
        }
    }
}
