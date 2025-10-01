//! A client for test

pub mod helper;
pub mod oc_helper;

pub use oc_helper::client::ClientCore;
pub use oc_helper::client::TestApp;
pub use oc_helper::user::TestUser;

#[ctor::ctor]
fn init() {
    dotenvy::dotenv().ok();
}
