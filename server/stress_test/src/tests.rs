pub mod auth;
pub mod basic;
pub mod file;
pub mod friend;
pub mod message;
pub mod negative;
pub mod registry;
pub mod session;
pub mod webrtc;

use std::sync::Arc;

pub type UsersGroup = Vec<Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>>;
