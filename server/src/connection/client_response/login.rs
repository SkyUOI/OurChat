use crate::consts::MessageType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub code: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocid: Option<String>,
    pub status: Status,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Error)]
#[repr(i32)]
pub enum Status {
    #[error("Success")]
    Success = 0,
    #[error("ServerError")]
    ServerError = 2,
    #[error("WrongPassword")]
    WrongPassword = 1,
}

impl LoginResponse {
    pub fn success_email(ocid: String) -> Self {
        Self {
            code: MessageType::LoginRes,
            ocid: Some(ocid),
            status: Status::Success,
        }
    }

    pub fn success_ocid() -> Self {
        Self {
            code: MessageType::LoginRes,
            ocid: None,
            status: Status::Success,
        }
    }

    pub fn failed(status: Status) -> Self {
        Self {
            code: MessageType::LoginRes,
            ocid: None,
            status,
        }
    }
}
