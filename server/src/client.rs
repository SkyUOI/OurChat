use tokio_tungstenite::tungstenite::Message;

pub mod basic;
pub mod requests;
pub mod response;

pub trait MsgConvert<'a>: serde::Deserialize<'a> {
    fn to_msg(&self) -> Message {
        Message::Text(self.to_json())
    }

    fn to_json(&self) -> String;

    fn from_json(json: &'a str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(serde_json::from_str(json)?)
    }
}

impl<'a, T> MsgConvert<'a> for T
where
    T: serde::Serialize + serde::Deserialize<'a>,
{
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
