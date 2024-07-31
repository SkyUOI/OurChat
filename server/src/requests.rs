//! 保存各种请求的结构体

pub mod login;
pub mod register;
pub mod unregister;

pub use login::{Login, LoginType};
pub use register::Register;
pub use unregister::Unregister;
