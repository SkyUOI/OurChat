use crate::client::basic::GetAccountValues;
use crate::consts::MessageType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountInfoResponse {
    pub code: MessageType,
    pub data: HashMap<GetAccountValues, serde_json::Value>,
}

impl GetAccountInfoResponse {
    pub fn success(data: HashMap<GetAccountValues, serde_json::Value>) -> Self {
        Self {
            code: MessageType::GetAccountInfoRes,
            data,
        }
    }
}
