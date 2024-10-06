use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Verify {
    pub code: MessageType,
    pub email: String,
}

impl Verify {
    pub fn new(email: String) -> Self {
        Self {
            code: MessageType::Verify,
            email,
        }
    }
}
