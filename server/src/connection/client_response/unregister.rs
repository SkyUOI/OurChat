//! 注销

use crate::consts;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
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
    pub fn new(status: Status) -> Self {
        Self {
            code: consts::MessageType::UnregisterRes,
            status,
        }
    }
}
