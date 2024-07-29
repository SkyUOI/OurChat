//! 保存各种请求的结构体

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    pub code: i32,
    pub time: u64,
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginType {
    Email = 0,
    Ocid = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub code: i32,
    pub time: u64,
    pub email: String,
    pub login_type: LoginType,
    pub password: String,
}
