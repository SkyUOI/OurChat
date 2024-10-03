//! Verification Response

use crate::{consts, requests};
use serde::{Deserialize, Serialize};

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

    pub fn email_cannot_be_sent() -> Self {
        Self {
            code: consts::MessageType::VerifyRes,
            status: requests::Status::UnknownInstruction,
        }
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
