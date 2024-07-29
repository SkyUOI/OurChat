use crate::{cfg::TimeStamp, consts::RequestType};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    code: RequestType,
    ocid: String,
    status: Status,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum Status {
    Success = 0,
    Dup = 2,
    Fail = 1,
}

impl RegisterResponse {
    pub fn new(ocid: String, status: Status) -> Self {
        Self {
            code: RequestType::RegisterRes,
            ocid,
            status,
        }
    }
}
