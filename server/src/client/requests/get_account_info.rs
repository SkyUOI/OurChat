use crate::consts::{MessageType, OCID};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::LazyLock};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum RequestValues {
    Ocid,
    Email,
    Nickname,
    Status,
    AvatarKey,
    Time,
    PublicUpdateTime,
    UpdateTime,
    Sessions,
    Friends,
}

pub static OWNER_PRIVILEGE: LazyLock<HashSet<RequestValues>> = LazyLock::new(|| {
    collection_literals::collection! {
        RequestValues::Sessions,
        RequestValues::Friends,
        RequestValues::UpdateTime,
        RequestValues::Email,
    }
});

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountInfoRequest {
    pub code: MessageType,
    pub ocid: OCID,
    pub request_values: Vec<RequestValues>,
}

impl GetAccountInfoRequest {
    pub fn new(ocid: OCID, request_values: Vec<RequestValues>) -> Self {
        Self {
            code: MessageType::GetAccountInfo,
            ocid,
            request_values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_account_info() {
        let request = GetAccountInfoRequest::new(OCID::from("test"), vec![RequestValues::Ocid]);
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            json,
            format!(
                "{{\"code\":{},\"ocid\":\"test\",\"request_values\":[\"ocid\"]}}",
                MessageType::GetAccountInfo as usize
            )
        );
    }
}
