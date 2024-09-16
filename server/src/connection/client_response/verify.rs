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
}
