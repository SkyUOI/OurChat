use crate::UsersGroup;
use crate::framework::{Report, run_user_stress_test};
use pb::service::ourchat::friends::accept_friend_invitation::v1::{
    AcceptFriendInvitationRequest, AcceptFriendInvitationResult,
};
use pb::service::ourchat::friends::add_friend::v1::AddFriendRequest;
use pb::service::ourchat::friends::delete_friend::v1::DeleteFriendRequest;
use pb::service::ourchat::friends::set_friend_info::v1::SetFriendInfoRequest;

use derive::register_test;

#[register_test("Add Friend", WithUsers)]
pub async fn test_add_friend(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "add_friend",
        users,
        100,
        100,
        |user, now, users| async move {
            let friend_idx = (now + 1) % users.len();
            let friend_id = users[friend_idx].lock().await.id;
            user.lock()
                .await
                .oc()
                .add_friend(AddFriendRequest {
                    friend_id: friend_id.0,
                    leave_message: Some("Stress test friend request".to_string()),
                    display_name: Some(format!("friend_{}", rand::random::<u32>())),
                })
                .await
                .is_ok()
        },
    )
    .await;
}

#[register_test("Accept Friend Invitation", WithUsers)]
pub async fn test_accept_friend_invitation(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "accept_friend_invitation",
        users,
        100,
        100,
        |user, now, users| async move {
            let friend_idx = if now == 0 { users.len() - 1 } else { now - 1 };
            let friend_id = users[friend_idx].lock().await.id;
            user.lock()
                .await
                .oc()
                .accept_friend_invitation(AcceptFriendInvitationRequest {
                    friend_id: friend_id.0,
                    status: AcceptFriendInvitationResult::Success.into(),
                    leave_message: Some("Accepted stress test".to_string()),
                })
                .await
                .is_ok()
        },
    )
    .await;
}

#[register_test("Delete Friend", WithUsers)]
pub async fn test_delete_friend(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "delete_friend",
        users,
        100,
        100,
        |user, now, users| async move {
            let friend_idx = (now + 1) % users.len();
            let friend_id = users[friend_idx].lock().await.id;
            user.lock()
                .await
                .oc()
                .delete_friend(DeleteFriendRequest {
                    friend_id: friend_id.0,
                })
                .await
                .is_ok()
        },
    )
    .await;
}

#[register_test("Set Friend Info", WithUsers)]
pub async fn test_set_friend_info(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "set_friend_info",
        users,
        100,
        100,
        |user, now, users| async move {
            let friend_idx = (now + 1) % users.len();
            let friend_id = users[friend_idx].lock().await.id;
            user.lock()
                .await
                .oc()
                .set_friend_info(SetFriendInfoRequest {
                    id: friend_id.0,
                    display_name: Some(format!("Updated friend {}", now)),
                })
                .await
                .is_ok()
        },
    )
    .await;
}
