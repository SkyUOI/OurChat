use crate::{
    client::basic::UnitMsg,
    consts::{MessageType, SessionID},
    entities::user_chat_msg,
};
use base::time::TimeStamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserMsgResponse {
    pub code: MessageType,
    pub session_id: SessionID,
    pub msgs: Vec<OneMsg>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OneMsg {
    pub session_id: SessionID,
    #[serde(with = "base::time::rfc3339")]
    pub time: TimeStamp,
    pub bundle_msg: Vec<UnitMsg>,
}

impl From<user_chat_msg::Model> for OneMsg {
    fn from(msg: user_chat_msg::Model) -> Self {
        Self {
            session_id: msg.session_id.into(),
            time: msg.time,
            bundle_msg: serde_json::from_value(msg.msg_data).unwrap(),
        }
    }
}

impl GetUserMsgResponse {
    // TODO: add this to document
    pub fn new(session_id: SessionID, msgs: Vec<OneMsg>) -> Self {
        Self {
            code: MessageType::GetBundledUserMsgRes,
            session_id,
            msgs,
        }
    }
}
