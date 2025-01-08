//! For communicating with http server

use serde::{Deserialize, Serialize};

pub const VERIFY_QUEUE: &str = "email_verify";

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct VerifyRecord {
    pub token: String,
    pub email: String,
}

impl VerifyRecord {
    pub fn new(email: String, token: String) -> Self {
        Self { token, email }
    }
}
