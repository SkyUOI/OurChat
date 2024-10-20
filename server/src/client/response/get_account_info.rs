use crate::client::basic::RequestValues;
use crate::{client::requests::Status, consts::MessageType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountInfoResponse {
    pub code: MessageType,
    pub data: Option<HashMap<RequestValues, serde_json::Value>>,
    pub status: Status,
}

impl GetAccountInfoResponse {
    pub fn success(data: HashMap<RequestValues, serde_json::Value>) -> Self {
        Self {
            code: MessageType::GetAccountInfoRes,
            data: Some(data),
            status: Status::Success,
        }
    }

    pub fn failure(status: Status) -> Self {
        Self {
            code: MessageType::GetAccountInfoRes,
            data: None,
            status,
        }
    }
}
