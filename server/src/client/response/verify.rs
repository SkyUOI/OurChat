//! Verification Response

use crate::{client::requests, consts};
use serde::{Deserialize, Serialize};

use super::ErrorMsgResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub code: consts::MessageType,
    pub status: requests::Status,
}

impl VerifyResponse {
    pub fn success() -> Self {
        Self {
            code: consts::MessageType::VerifyRes,
            status: requests::Status::Success,
        }
    }

    pub fn email_cannot_be_sent() -> ErrorMsgResponse {
        ErrorMsgResponse::new(requests::Status::FeatureDisable, "Email Cannot Be Sent")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyStatusResponse {
    pub code: consts::MessageType,
    pub status: requests::Status,
}

impl VerifyStatusResponse {
    pub fn success() -> Self {
        Self {
            code: consts::MessageType::VerifyStatusRes,
            status: requests::Status::Success,
        }
    }
}
