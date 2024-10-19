//! define functions process the requests from clients directly

mod get_account_info;
pub mod login;
pub mod new_session;
pub mod register;
mod set_account_info;
pub mod unregister;
mod upload;
pub mod verify;

pub use get_account_info::get_account_info;
pub use new_session::accept_session;
pub use new_session::new_session;
pub use register::register;
pub use unregister::unregister;
pub use upload::upload;
