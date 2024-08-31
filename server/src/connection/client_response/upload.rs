//! 上传文件

use crate::{consts, requests::Status};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub code: consts::MessageType,
    status: Status,
    url: Option<String>,
    key: Option<String>,
    hash: Option<String>,
}

impl UploadResponse {
    pub fn success(url: String, key: String, hash: String) -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            url: Some(url),
            key: Some(key),
            hash: Some(hash),
            status: Status::Success,
        }
    }

    pub fn limited() -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            url: None,
            key: None,
            hash: None,
            status: Status::AccountLimitation,
        }
    }
}
