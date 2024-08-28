use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Upload {
    pub code: MessageType,
    pub hash: String,
}

impl Upload {
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            code: MessageType::Upload,
            hash: hash.into(),
        }
    }
}
