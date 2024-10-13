use crate::consts::MessageType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum LoginType {
    Email = 0,
    Ocid = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub code: MessageType,
    pub account: String,
    pub login_type: LoginType,
    pub password: String,
}

impl LoginRequest {
    pub fn new(account: String, password: String, login_type: LoginType) -> Self {
        Self {
            code: MessageType::Login,
            account,
            password,
            login_type,
        }
    }
}
