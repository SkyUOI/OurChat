use crate::consts::MessageType;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMsgResponse {
    pub code: MessageType,
    pub details: String,
}

impl ErrorMsgResponse {
    pub fn new(details: String) -> Self {
        Self {
            code: MessageType::ErrorMsg,
            details,
        }
    }
}
