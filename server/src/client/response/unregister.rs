//! Unregister Response

use crate::consts;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub code: consts::MessageType,
}

impl UnregisterResponse {
    pub fn new() -> Self {
        Self {
            code: consts::MessageType::UnregisterRes,
        }
    }
}
