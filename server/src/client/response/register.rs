use crate::{client::requests::Status, consts::MessageType};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub code: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocid: Option<String>,
    pub status: Status,
}

impl RegisterResponse {
    pub fn success(ocid: String) -> Self {
        Self {
            code: MessageType::RegisterRes,
            ocid: Some(ocid),
            status: Status::Success,
        }
    }

    pub fn failed(status: Status) -> Self {
        Self {
            code: MessageType::RegisterRes,
            ocid: None,
            status,
        }
    }
}
