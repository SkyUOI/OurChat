use tokio_tungstenite::tungstenite::Message;

pub mod basic;
pub mod requests;
pub mod response;

pub trait MsgConvert {
    fn to_msg(&self) -> Message {
        Message::Text(self.to_json())
    }

    fn to_json(&self) -> String;
}

impl<T> MsgConvert for T
where
    T: serde::Serialize,
{
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
