use crate::consts::MessageType;
use base::time::TimeStamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserMsgRequest {
    pub code: MessageType,
    #[serde(with = "base::time::rfc3339")]
    pub time: TimeStamp,
}

impl GetUserMsgRequest {
    pub fn new(time: TimeStamp) -> Self {
        Self {
            code: MessageType::GetUserMsg,
            time,
        }
    }
}
