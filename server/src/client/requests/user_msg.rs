use crate::consts::{MessageType, TimeStamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMsg {
    pub code: MessageType,
    pub time: TimeStamp,
}

impl UserMsg {
    pub fn new(time: TimeStamp) -> Self {
        Self {
            code: MessageType::UserMsg,
            time,
        }
    }
}
