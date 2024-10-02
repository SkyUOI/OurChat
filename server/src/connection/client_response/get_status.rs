//! 注销返回信息

use crate::{consts, requests};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub code: consts::MessageType,
    pub status: requests::Status,
}

impl GetStatusResponse {
    pub fn normal() -> Self {
        Self {
            code: consts::MessageType::GetStatus,
            status: requests::Status::Success,
        }
    }

    pub fn maintaining() -> Self {
        Self {
            code: consts::MessageType::GetStatus,
            status: requests::Status::Maintaining,
        }
    }
}
