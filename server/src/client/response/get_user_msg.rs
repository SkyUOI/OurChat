use crate::{
    client::basic::Msg,
    consts::{MessageType, SessionID},
};
use base::time::TimeStamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserMsgResponse {
    pub code: MessageType,
    pub session_id: SessionID,
    pub msgs: Vec<OneMsg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneMsg {
    pub session_id: SessionID,
    #[serde(with = "base::time::rfc3339")]
    pub time: TimeStamp,
    pub bundle_msg: Vec<Msg>,
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
