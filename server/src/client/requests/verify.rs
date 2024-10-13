use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub code: MessageType,
    pub email: String,
}

impl VerifyRequest {
    pub fn new(email: String) -> Self {
        Self {
            code: MessageType::Verify,
            email,
        }
    }
}
