//! define functions process the requests from clients directly

pub mod login;
pub mod new_session;
pub mod register;
pub mod unregister;
mod upload;
pub mod verify;

pub use new_session::accept_session;
pub use new_session::new_session;
pub use register::register;
pub use unregister::unregister;
pub use upload::upload;
