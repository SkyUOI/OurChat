use crate::{
    entities::user_chat_msg,
    utils::{from_google_timestamp, to_google_timestamp},
};

tonic::include_proto!("msg_delivery");

pub type BundleMsgs = Vec<OneMsg>;

impl TryFrom<user_chat_msg::Model> for Msg {
    type Error = anyhow::Error;

    fn try_from(msg: user_chat_msg::Model) -> Result<Self, anyhow::Error> {
        Ok(Self {
            msg_id: msg.chat_msg_id.try_into()?,
            bundle_msg: serde_json::from_value(msg.msg_data)?,
            session_id: msg.session_id.try_into()?,
            time: Some(to_google_timestamp(msg.time.into())),
        })
    }
}
