pub mod auth;
pub mod basic;
pub mod file;
pub mod friend;
pub mod message;
pub mod negative;
pub mod session;
pub mod webrtc;

use std::sync::Arc;

pub type UsersGroup = Vec<Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>>;

pub use auth::{test_auth, test_get_info, test_register, test_set_account_info, test_unregister};
pub use basic::{test_basic_service, test_get_id, test_preset_user_status};
pub use file::{test_download, test_upload};
pub use friend::{
    test_accept_friend_invitation, test_add_friend, test_delete_friend, test_set_friend_info,
};
pub use message::{test_fetch_msgs, test_recall, test_send_msg};
pub use negative::{
    test_add_friend_invalid_user, test_delete_session_unauthorized, test_get_session_info_invalid,
    test_join_nonexistent_session, test_send_empty_message, test_send_msg_invalid_session,
};
pub use session::{
    test_accept_join_session_invitation, test_add_role, test_allow_user_join_session, test_ban,
    test_dee2eeize_session, test_delete_session, test_e2eeize_session, test_get_role,
    test_get_session_info, test_invite_user_to_session, test_join_session, test_kick,
    test_leave_session, test_mute, test_new_session, test_send_room_key, test_set_role,
    test_set_session_info,
};
pub use webrtc::test_create_room;
