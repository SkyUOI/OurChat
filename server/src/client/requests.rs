//! Requests from client to server

mod accept_session;
pub mod new_session;
mod set_session_info;
mod unregister;
pub mod upload;
mod user_send_msg;
mod verify;

pub use accept_session::AcceptSessionRequest;
pub use new_session::NewSessionRequest;
pub use unregister::UnregisterRequest;
pub use user_send_msg::UserSendMsgRequest;
pub use verify::VerifyRequest;

use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;

#[derive(Debug, Serialize_repr, Deserialize_repr, Error, PartialEq, Eq)]
#[repr(u32)]
pub enum Status {
    // basic define
    #[error("Success")]
    Success = 0,
    #[error("Server Error")]
    ServerError = 1,
    #[error("Maintaining")]
    Maintaining = 2,
    #[error("Request Info Not Found")]
    RequestInfoNotFound = 3,
    #[error("Info Exists")]
    InfoExists = 4,
    #[error("Argument or Instruction NotFound Error")]
    ArgOrInstNotCorrectError,
    #[error("Account Limitation")]
    AccountLimitation = 6,
    #[error("Timeout")]
    Timeout = 7,
    #[error("Unknown Error")]
    UnknownError = 8,
    #[error("Reject")]
    RequestReject = 9,
    #[error("Verify Failed")]
    VerifyFailed = 10,
    #[error("Feature Disable")]
    FeatureDisable = 11,
}
