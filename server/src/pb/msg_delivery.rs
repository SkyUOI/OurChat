use crate::entities::user_chat_msg;

tonic::include_proto!("msg_delivery");

impl From<user_chat_msg::Model> for Msg {
    fn from(msg: user_chat_msg::Model) -> Self {
        todo!()
    }
}
