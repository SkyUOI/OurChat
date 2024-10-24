//! Unregister Response

use crate::{client::requests, consts};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub code: consts::MessageType,
    pub status: requests::Status,
}

impl UnregisterResponse {
    pub fn new(status: requests::Status) -> Self {
        Self {
            code: consts::MessageType::UnregisterRes,
            status,
        }
    }
}
