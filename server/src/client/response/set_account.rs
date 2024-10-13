use crate::client::requests::Status;
use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetAccountResponse {
    pub code: MessageType,
    pub status: Status,
}

impl SetAccountResponse {
    pub fn new(status: Status) -> Self {
        Self {
            code: MessageType::SetAccountRes,
            status,
        }
    }
}
