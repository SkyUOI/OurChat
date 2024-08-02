use crate::consts::MessageType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub code: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocid: Option<String>,
    pub status: Status,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Error, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    #[error("success")]
    Success = 0,
    #[error("dup")]
    Dup = 2,
    #[error("fail")]
    ServerError = 1,
}

impl RegisterResponse {
    pub fn success(ocid: String) -> Self {
        Self {
            code: MessageType::RegisterRes,
            ocid: Some(ocid),
            status: Status::Success,
        }
    }

    pub fn failed(status: Status) -> Self {
        Self {
            code: MessageType::RegisterRes,
            ocid: None,
            status,
        }
    }
}
