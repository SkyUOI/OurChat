//! Upload Files

use crate::client::response::ErrorMsgResponse;
use crate::{client::requests::Status, consts};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub code: consts::MessageType,
    pub key: String,
    pub hash: String,
}

impl UploadResponse {
    pub fn success(key: String, hash: String) -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            key,
            hash,
        }
    }

    pub fn limited() -> ErrorMsgResponse {
        ErrorMsgResponse::new(
            Status::AccountLimitation,
            "Account Limitation Has Been Reached".to_string(),
        )
    }
}
