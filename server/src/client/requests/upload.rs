use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadRequest {
    pub code: MessageType,
    pub hash: String,
    pub auto_clean: bool,
    pub size: u64,
}

impl UploadRequest {
    pub fn new(hash: impl Into<String>, auto_clean: bool, size: u64) -> Self {
        Self {
            code: MessageType::Upload,
            hash: hash.into(),
            auto_clean,
            size,
        }
    }
}
