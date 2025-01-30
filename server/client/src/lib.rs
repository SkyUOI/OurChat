#![feature(os_str_display)]

//! A client for test

pub mod helper;
pub mod http_helper;
pub mod oc_helper;

pub use http_helper::TestHttpApp;
pub use oc_helper::client::TestApp;
pub use oc_helper::user::TestUser;

#[ctor::ctor]
fn init() {
    helper::init_env_var();
}
