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
mod invite_session;
mod new_session;
mod opers;
mod upload;
mod user_send_msg;
pub mod verify;

pub use accept_session::AcceptSessionResponse;
pub use invite_session::InviteSession;
pub use new_session::NewSessionResponse;
pub use opers::OpersResponse;
pub use upload::UploadResponse;
pub use user_send_msg::UserSendMsgResponse;
pub use verify::VerifyResponse;
pub use verify::VerifyStatusResponse;
