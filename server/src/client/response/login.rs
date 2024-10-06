use crate::{client::requests::Status, consts::MessageType};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub code: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocid: Option<String>,
    pub status: Status,
}

pub macro Status(WrongPassword) {
    $crate::client::requests::Status::ArgumentError
}

impl LoginResponse {
    pub fn success_email(ocid: String) -> Self {
        Self {
            code: MessageType::LoginRes,
            ocid: Some(ocid),
            status: Status::Success,
        }
    }

    pub fn success_ocid() -> Self {
        Self {
            code: MessageType::LoginRes,
            ocid: None,
            status: Status::Success,
        }
    }

    pub fn failed(status: Status) -> Self {
        Self {
            code: MessageType::LoginRes,
            ocid: None,
            status,
        }
    }
}

impl From<LoginResponse> for Message {
    fn from(value: LoginResponse) -> Self {
        Message::Text(serde_json::to_string(&value).unwrap())
    }
}
