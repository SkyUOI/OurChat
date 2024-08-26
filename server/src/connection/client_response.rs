//! 返回给客户端的结果

pub mod error_msg;
pub mod get_status;
pub mod login;
pub mod new_session;
pub mod register;
pub mod unregister;

pub use error_msg::ErrorMsgResponse;
pub use login::LoginResponse;
pub use new_session::NewSessionResponse;
pub use register::RegisterResponse;
pub use unregister::UnregisterResponse;
