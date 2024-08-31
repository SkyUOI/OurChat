use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Upload {
    pub code: MessageType,
    pub hash: String,
    pub auto_clean: bool,
}

impl Upload {
    pub fn new(hash: impl Into<String>, auto_clean: bool) -> Self {
        Self {
            code: MessageType::Upload,
            hash: hash.into(),
            auto_clean,
        }
    }
}
