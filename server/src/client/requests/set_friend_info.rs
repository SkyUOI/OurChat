use crate::{
    client::basic::SetFriendValues,
    consts::{MessageType, OCID},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Data = HashMap<SetFriendValues, serde_json::Value>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetFriendInfoRequest {
    pub code: MessageType,
    pub ocid: OCID,
    pub data: Data,
}

impl SetFriendInfoRequest {
    pub fn new(ocid: OCID, data: Data) -> Self {
        Self {
            code: MessageType::SetFriendInfo,
            ocid,
            data,
        }
    }
}
