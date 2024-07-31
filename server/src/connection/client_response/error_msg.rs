use crate::consts::{self, RequestType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMsgResponse {
    pub code: RequestType,
    pub details: String,
}

impl ErrorMsgResponse {
    pub fn new(details: String) -> Self {
        Self {
            code: consts::RequestType::ErrorMsg,
            details,
        }
    }
}
