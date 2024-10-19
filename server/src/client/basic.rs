use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RequestValues {
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
