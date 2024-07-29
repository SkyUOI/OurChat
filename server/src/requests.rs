//! 保存各种请求的结构体

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Register {
    code: i32,
    time: u64,
    email: String,
    password: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum LoginType {
    Email = 0,
    Ocid = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    code: i32,
    time: u64,
    email: String,
    login_type: LoginType,
    password: String,
}
