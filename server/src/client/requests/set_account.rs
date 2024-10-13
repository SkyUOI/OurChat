use std::collections::HashMap;

use crate::consts::MessageType;

pub struct SetAccountRequest {
    pub code: MessageType,
    pub data: HashMap<String, serde_json::Value>,
}

impl SetAccountRequest {
    pub fn new(data: HashMap<String, serde_json::Value>) -> Self {
        Self {
            code: MessageType::SetAccount,
            data,
        }
    }
}
