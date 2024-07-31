//! 返回给客户端的结果

pub mod error_msg;
pub mod login;
pub mod register;
pub mod unregister;

pub use error_msg::ErrorMsgResponse;
pub use login::LoginResponse;
pub use register::RegisterResponse;
pub use unregister::UnregisterResponse;
