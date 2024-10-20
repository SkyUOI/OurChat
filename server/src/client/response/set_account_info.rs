use crate::client::requests::Status;
use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetAccountInfoResponse {
    pub code: MessageType,
    pub status: Status,
}

impl SetAccountInfoResponse {
    pub fn success() -> Self {
        Self {
            code: MessageType::SetAccountInfoRes,
            status: Status::Success,
        }
    }

    pub fn arg_error() -> Self {
        Self {
            code: MessageType::SetAccountInfoRes,
            status: Status::ArgumentError,
        }
    }

    pub fn server_error() -> Self {
        Self {
            code: MessageType::SetAccountInfoRes,
            status: Status::ServerError,
        }
    }
}
