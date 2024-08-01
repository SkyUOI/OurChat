use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Unregister {
    pub code: MessageType,
}

impl Unregister {
    pub fn new() -> Self {
        Self {
            code: MessageType::Unregister,
        }
    }
}
