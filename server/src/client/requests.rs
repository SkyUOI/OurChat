//! Requests from client to server

mod accept_session;
pub mod get_account_info;
mod get_status;
mod get_user_msg;
mod login;
pub mod new_session;
mod register;
pub mod set_account_info;
mod set_friend_info;
mod set_session_info;
mod unregister;
pub mod upload;
mod user_send_msg;
mod verify;

pub use accept_session::AcceptSessionRequest;
pub use get_account_info::GetAccountInfoRequest;
pub use get_status::GetStatus;
pub use get_user_msg::GetUserMsgRequest;
pub use login::{LoginRequest, LoginType};
pub use new_session::NewSessionRequest;
pub use register::RegisterRequest;
pub use set_account_info::SetAccountRequest;
pub use set_friend_info::SetFriendInfoRequest;
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
