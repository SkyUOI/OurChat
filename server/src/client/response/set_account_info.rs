use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetAccountInfoResponse {
    pub code: MessageType,
}

impl SetAccountInfoResponse {
    pub fn success() -> Self {
        Self {
            code: MessageType::SetAccountInfoRes,
        }
    }
}
