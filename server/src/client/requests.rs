//! Requests from client to server

pub mod get_status;
pub mod login;
pub mod new_session;
pub mod register;
pub mod unregister;
pub mod upload;
pub mod user_msg;
pub mod verify;

pub use login::{Login, LoginType};
pub use new_session::NewSession;
pub use register::Register;
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;
pub use unregister::Unregister;
pub use verify::Verify;

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
    #[error("Unknown Instruction")]
    UnknownInstruction = 3,
    #[error("Dup")]
    Dup = 4,
    #[error("Argument Error")]
    ArgumentError = 5,
    #[error("Account Limitation")]
    AccountLimitation = 6,
    #[error("Timeout")]
    Timeout = 7,
    #[error("Unknown Error")]
    UnknownError = 8,
}
