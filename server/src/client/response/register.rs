use crate::{client::requests::Status, consts::MessageType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub code: MessageType,
    pub ocid: String,
    pub status: Status,
}

impl RegisterResponse {
    pub fn success(ocid: String) -> Self {
        Self {
            code: MessageType::RegisterRes,
            ocid,
            status: Status::Success,
        }
    }
}
