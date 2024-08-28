//! 上传文件

use crate::consts;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub code: consts::MessageType,
    url: String,
    key: String,
}

impl UploadResponse {
    pub fn new(url: String, key: String) -> Self {
        Self {
            code: consts::MessageType::UploadRes,
            url,
            key,
        }
    }
}
