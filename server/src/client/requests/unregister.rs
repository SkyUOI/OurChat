use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterRequest {
    pub code: MessageType,
}

impl UnregisterRequest {
    pub fn new() -> Self {
        Self {
            code: MessageType::Unregister,
        }
    }
}

impl Default for UnregisterRequest {
    fn default() -> Self {
        Self::new()
    }
}
