use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatus {
    pub code: MessageType,
}

impl GetStatus {
    pub fn new() -> Self {
        Self {
            code: MessageType::GetStatus,
        }
    }
}

impl Default for GetStatus {
    fn default() -> Self {
        Self::new()
    }
}
