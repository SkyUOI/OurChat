//! Upload Files

use crate::{client::requests::Status, consts};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub code: consts::MessageType,
    status: Status,
    key: Option<String>,
    hash: Option<String>,
}

impl UploadResponse {
    pub fn success(key: String, hash: String) -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            key: Some(key),
            hash: Some(hash),
            status: Status::Success,
        }
    }

    pub fn limited() -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            key: None,
            hash: None,
            status: Status::AccountLimitation,
        }
    }
}
