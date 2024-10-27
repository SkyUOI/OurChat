use crate::consts::{MessageType, MsgID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSendMsgResponse {
    pub code: MessageType,
    pub msg_id: MsgID,
}

impl UserSendMsgResponse {
    pub fn success(msg_id: MsgID) -> Self {
        Self {
            code: MessageType::UserSendMsgRes,
            msg_id,
        }
    }
}
