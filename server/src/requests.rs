//! 保存各种请求的结构体

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub code: i32,
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum LoginType {
    Email = 0,
    Ocid = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub code: i32,
    pub account: String,
    pub login_type: LoginType,
    pub password: String,
}
