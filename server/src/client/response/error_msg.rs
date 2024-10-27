use crate::client::requests::Status;
use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMsgResponse {
    pub code: MessageType,
    pub status: Status,
    pub details: String,
}

impl ErrorMsgResponse {
    pub fn new(status: Status, details: impl Into<String>) -> Self {
        Self {
            code: MessageType::ErrorMsg,
            status,
            details: details.into(),
        }
    }

    pub fn server_error(details: impl Into<String>) -> Self {
        Self {
            code: MessageType::ErrorMsg,
            status: Status::ServerError,
            details: details.into(),
        }
    }
}
