use crate::client::basic::GetAccountValues;
use crate::consts::{MessageType, OCID};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::LazyLock};

pub static OWNER_PRIVILEGE: LazyLock<HashSet<GetAccountValues>> = LazyLock::new(|| {
    collection_literals::collection! {
        GetAccountValues::Sessions,
        GetAccountValues::Friends,
        GetAccountValues::UpdateTime,
        GetAccountValues::Email,
    }
});

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAccountInfoRequest {
    pub code: MessageType,
    pub ocid: OCID,
    pub request_values: Vec<GetAccountValues>,
}

impl GetAccountInfoRequest {
    pub fn new(ocid: OCID, request_values: Vec<GetAccountValues>) -> Self {
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
        let request = GetAccountInfoRequest::new(OCID::from("test"), vec![GetAccountValues::Ocid]);
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
