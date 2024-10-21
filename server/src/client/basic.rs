use serde::{Deserialize, Serialize};

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
