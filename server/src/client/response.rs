//! Response to client from server

// # Template for Developers
// ```
// use crate::{client::requests::Status, consts::MessageType};
// use serde::{Deserialize, Serialize};
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct xxxResponse {
//     pub code: MessageType,
// }
//
// impl xxxResponse {
//     pub fn new() -> Self {
//         Self {
//             code: MessageType::AcceptSessionRes,
//         }
//     }
// }
// ```

mod accept_session;
pub mod error_msg;
mod get_account_info;
pub mod get_status;
mod get_user_msg;
mod invite_session;
pub mod login;
mod new_session;
mod opers;
mod register;
mod set_account_info;
mod unregister;
mod upload;
mod user_send_msg;
pub mod verify;

pub use accept_session::AcceptSessionResponse;
pub use error_msg::ErrorMsgResponse;
pub use get_account_info::GetAccountInfoResponse;
pub use get_user_msg::{GetUserMsgResponse, OneMsg};
pub use invite_session::InviteSession;
pub use login::LoginResponse;
pub use new_session::NewSessionResponse;
pub use opers::OpersResponse;
pub use register::RegisterResponse;
pub use set_account_info::SetAccountInfoResponse;
pub use unregister::UnregisterResponse;
pub use upload::UploadResponse;
pub use user_send_msg::UserSendMsgResponse;
pub use verify::VerifyResponse;
pub use verify::VerifyStatusResponse;
