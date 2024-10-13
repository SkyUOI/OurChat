use crate::consts::MessageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub code: MessageType,
    pub email: String,
    pub password: String,
    pub name: String,
}

impl RegisterRequest {
    pub fn new(name: String, password: String, email: String) -> Self {
        Self {
            code: MessageType::Register,
            name,
            password,
            email,
        }
    }
}
