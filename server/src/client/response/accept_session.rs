//! Accept session response

use crate::{client::requests::Status, consts::MessageType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptSessionResponse {
    pub code: MessageType,
    pub status: Status,
}

impl AcceptSessionResponse {
    pub fn success() -> Self {
        Self {
            code: MessageType::AcceptSessionRes,
            status: Status::Success,
        }
    }

    pub fn failed() -> Self {
        Self {
            code: MessageType::AcceptSessionRes,
            status: Status::AccountLimitation,
        }
    }
}
