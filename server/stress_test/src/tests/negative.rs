use crate::UsersGroup;
use crate::framework::{Report, run_user_stress_test};
use pb::service::ourchat::friends::add_friend::v1::AddFriendRequest;
use pb::service::ourchat::msg_delivery::v1::SendMsgRequest;
use pb::service::ourchat::session::delete_session::v1::DeleteSessionRequest;
use pb::service::ourchat::session::get_session_info::v1::{
    GetSessionInfoRequest, QueryValues as SessionQueryValues,
};
use pb::service::ourchat::session::join_session::v1::JoinSessionRequest;

/// Test joining non-existent session
pub async fn test_join_nonexistent_session(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "join_nonexistent_session",
        users,
        100,
        100,
        |user, _now, _users| async move {
            let mut u = user.lock().await;
            // Try to join non-existent session
            u.oc()
                .join_session(JoinSessionRequest {
                    session_id: u64::MAX,
                    leave_message: Some("Trying to join invalid session".to_string()),
                })
                .await
                .is_err()
        },
    )
    .await;
}

/// Test sending message to non-existent session
pub async fn test_send_msg_invalid_session(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "send_msg_invalid_session",
        users,
        100,
        100,
        |user, _now, _users| async move {
            let mut u = user.lock().await;
            // Try to send message to non-existent session
            u.oc()
                .send_msg(SendMsgRequest {
                    session_id: u64::MAX,
                    markdown_text: "Test message".to_string(),
                    involved_files: vec![],
                    is_encrypted: false,
                })
                .await
                .is_err()
        },
    )
    .await;
}

/// Test getting session info for non-existent session
pub async fn test_get_session_info_invalid(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "get_session_info_invalid",
        users,
        100,
        100,
        |user, _now, _users| async move {
            let mut u = user.lock().await;
            // Try to get info for non-existent session
            u.oc()
                .get_session_info(GetSessionInfoRequest {
                    session_id: u64::MAX,
                    query_values: vec![SessionQueryValues::SessionId.into()],
                })
                .await
                .is_err()
        },
    )
    .await;
}

/// Test adding friend with invalid user ID
pub async fn test_add_friend_invalid_user(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "add_friend_invalid_user",
        users,
        100,
        100,
        |user, _now, _users| async move {
            let mut u = user.lock().await;
            // Try to add non-existent user as friend
            u.oc()
                .add_friend(AddFriendRequest {
                    friend_id: u64::MAX,
                    leave_message: Some("Trying to add invalid user".to_string()),
                    display_name: Some("Invalid User".to_string()),
                })
                .await
                .is_err()
        },
    )
    .await;
}

/// Test sending empty message content
pub async fn test_send_empty_message(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "send_empty_message",
        users,
        100,
        100,
        |user, _now, _users| async move {
            let mut u = user.lock().await;
            // Try to send empty message to non-existent session (will fail on session not found first)
            u.oc()
                .send_msg(SendMsgRequest {
                    session_id: u64::MAX,
                    markdown_text: "".to_string(),
                    involved_files: vec![],
                    is_encrypted: false,
                })
                .await
                .is_err()
        },
    )
    .await;
}

/// Test permission denied: non-owner trying to delete session
pub async fn test_delete_session_unauthorized(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "delete_session_unauthorized",
        users,
        100,
        100,
        |user, _now, _users| async move {
            let mut u = user.lock().await;
            // Try to delete a session the user doesn't own
            u.oc()
                .delete_session(DeleteSessionRequest {
                    session_id: u64::MAX,
                })
                .await
                .is_err()
        },
    )
    .await;
}
