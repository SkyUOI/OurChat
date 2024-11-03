use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GetAccountValues {
    Ocid,
    Email,
    DisplayName,
    UserName,
    Status,
    AvatarKey,
    Time,
    PublicUpdateTime,
    UpdateTime,
    Sessions,
    Friends,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SetAccountValues {
    UserName,
    AvatarKey,
    Status,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SetFriendValues {
    DisplayName,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum SetSessionValues {
    DisplayName,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum UserMsgType {
    Text = 0,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TextMsg {
    pub r#type: UserMsgType,
    pub text: String,
}

impl TextMsg {
    pub fn new(text: String) -> Self {
        Self {
            r#type: UserMsgType::Text,
            text,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UnitMsg {
    Text(TextMsg),
}
