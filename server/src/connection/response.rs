//! 服务器间处理请求的异常

use std::fmt::Display;

use thiserror::Error;

#[derive(Debug)]
pub enum LoginError {}

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("already exists")]
    AlreadyExists,
    #[error("server error")]
    ServerError(String),
}
