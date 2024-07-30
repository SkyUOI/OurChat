use crate::consts::RequestType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    code: RequestType,
    ocid: String,
    status: Status,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum Status {
    Success = 0,
    ServerError = 2,
    WrongPassword = 1,
}

impl LoginResponse {
    pub fn new(ocid: String, status: Status) -> Self {
        Self {
            code: RequestType::LoginRes,
            ocid,
            status,
        }
    }
}
