use base::consts::SessionID;
use claims::{assert_none, assert_some};
use client::TestApp;
use migration::m20241229_022701_add_role_for_session::PredefinedRoles;
use pb::service::ourchat::friends::accept_friend::v1::{AcceptFriendRequest, AcceptFriendResult};
use pb::service::ourchat::friends::add_friend::v1::AddFriendRequest;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use server::db::friend::query_friend;
use server::db::session::get_all_roles_of_session;
use server::process::error_msg;
use std::time::Duration;

/// Tests the process of adding and accepting a friend request between two users.
///
/// This test performs the following actions:
/// 1. User1 sends a friend request to User2 with a display name and no leave message.
/// 2. Verifies that User2 receives the friend request and checks the details of the request.
/// 3. User2 accepts the friend request from User1, resulting in a session creation.
/// 4. Confirms the existence of the friendship in the database and checks the messages received by User1.
/// 5. Ensures that both users are assigned the "Owner" role in the new session.
#[tokio::test]
async fn add_friend_accept() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let (user1_id, user2_id) = (user1.lock().await.id, user2.lock().await.id);
    user1
        .lock()
        .await
        .oc()
        .add_friend(AddFriendRequest {
            friend_id: user2_id.into(),
            leave_message: None,
            display_name: Some("new_friend".to_owned()),
        })
        .await
        .unwrap();
    let user2_rec = user2.lock().await.fetch_msgs().fetch(1).await.unwrap();
    let RespondEventType::AddFriendApproval(add_friend) =
        user2_rec[0].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(add_friend.leave_message, None);
    assert_eq!(add_friend.inviter_id, *user1_id);

    let send_invitation = async || {
        user2
            .lock()
            .await
            .oc()
            .accept_friend(AcceptFriendRequest {
                friend_id: user1_id.into(),
                leave_message: None,
                status: AcceptFriendResult::Success.into(),
            })
            .await
    };
    let ret = send_invitation().await.unwrap().into_inner();
    let session_id1: SessionID = ret.session_id.unwrap().into();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_some!(
        query_friend(user1_id, user2_id, app.get_db_connection())
            .await
            .unwrap()
    );
    let user1_rec = user1.lock().await.fetch_msgs().fetch(2).await.unwrap();
    let RespondEventType::AcceptFriend(accept_friend_notification) =
        user1_rec[1].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(accept_friend_notification.leave_message, None);
    assert_eq!(accept_friend_notification.inviter_id, *user1_id);
    assert_eq!(accept_friend_notification.invitee_id, *user2_id);
    assert_eq!(
        accept_friend_notification.status,
        AcceptFriendResult::Success as i32
    );
    let session_id2: SessionID = accept_friend_notification.session_id.unwrap().into();
    assert_eq!(session_id1, session_id2);
    let members = get_all_roles_of_session(session_id1, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(members.len(), 2);
    for i in &members {
        assert_eq!(i.role_id, PredefinedRoles::Owner as i64);
    }
    // test that the invitation is not sent again
    let err = send_invitation().await.unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::FRIEND_INVITATION);
    app.async_drop().await;
}

/// Test that a friend request can be rejected.
///
/// 1. User1 sends a friend request to User2.
/// 2. User2 rejects the friend request.
/// 3. Verify that the friend relation isn't created.
/// 4. Verify that User1 and User2 each receive the corresponding notification.
#[tokio::test]
async fn add_friend_reject() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let (user1_id, user2_id) = (user1.lock().await.id, user2.lock().await.id);
    user1
        .lock()
        .await
        .oc()
        .add_friend(AddFriendRequest {
            friend_id: user2_id.into(),
            leave_message: None,
            display_name: Some("new_friend".to_owned()),
        })
        .await
        .unwrap();
    let user2_rec = user2.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(user2_rec.len(), 1);
    let RespondEventType::AddFriendApproval(add_friend) =
        user2_rec[0].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(add_friend.leave_message, None);
    assert_eq!(add_friend.inviter_id, *user1_id);
    let send_invitation = async || {
        user2
            .lock()
            .await
            .oc()
            .accept_friend(AcceptFriendRequest {
                friend_id: user1_id.into(),
                leave_message: None,
                status: AcceptFriendResult::Fail.into(),
            })
            .await
    };
    let ret = send_invitation().await.unwrap().into_inner();
    assert_none!(ret.session_id);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_none!(
        query_friend(user1_id, user2_id, app.get_db_connection())
            .await
            .unwrap()
    );
    let user1_rec = user1.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(user1_rec.len(), 2, "{user1_rec:?}");
    let RespondEventType::AcceptFriend(accept_friend_notification) =
        user1_rec[1].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(accept_friend_notification.leave_message, None);
    assert_eq!(accept_friend_notification.inviter_id, *user1_id);
    assert_eq!(accept_friend_notification.invitee_id, *user2_id);
    assert_eq!(
        accept_friend_notification.status,
        AcceptFriendResult::Fail as i32
    );
    assert_none!(accept_friend_notification.session_id);

    // test that the invitation is not sent again
    let err = send_invitation().await.unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::FRIEND_INVITATION);
    app.async_drop().await;
}
