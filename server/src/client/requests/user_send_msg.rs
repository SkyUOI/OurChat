use crate::{
    client::basic::Msg,
    consts::{MessageType, SessionID},
};
use base::time::TimeStamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSendMsgRequest {
    pub code: MessageType,
    pub session_id: SessionID,
    #[serde(with = "base::time::rfc3339")]
    pub time: TimeStamp,
    pub bundle_msg: Vec<Msg>,
}

impl UserSendMsgRequest {
    pub fn new(session_id: SessionID, time: TimeStamp, bundle_msg: Vec<Msg>) -> Self {
        Self {
            code: MessageType::UserSendMsg,
            session_id,
            time,
            bundle_msg,
        }
    }
}
