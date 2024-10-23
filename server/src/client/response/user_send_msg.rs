use crate::{
    client::requests::Status,
    consts::{MessageType, MsgID},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSendMsgResponse {
    pub code: MessageType,
    pub status: Status,
    pub msg_id: Option<MsgID>,
}

impl UserSendMsgResponse {
    pub fn success(msg_id: MsgID) -> Self {
        Self {
            code: MessageType::UserSendMsgRes,
            status: Status::Success,
            msg_id: Some(msg_id),
        }
    }

    pub fn failure(status: Status) -> Self {
        Self {
            code: MessageType::UserSendMsgRes,
            status,
            msg_id: None,
        }
    }
}
