use base::consts::SessionID;
use claims::{assert_none, assert_some};
use client::TestApp;
use migration::m20241229_022701_add_role_for_session::PredefinedRoles;
use pb::service::ourchat::friends::accept_friend::v1::{AcceptFriendRequest, AcceptFriendResult};
use pb::service::ourchat::friends::add_friend::v1::AddFriendRequest;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use server::db::friend::query_friend;
use server::db::session::get_all_roles_of_session;
use std::time::Duration;

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
    let user2_rec = user2
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(600))
        .await
        .unwrap();
    assert_eq!(user2_rec.len(), 1);
    let RespondMsgType::AddFriendApproval(add_friend) =
        user2_rec[0].respond_msg_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(add_friend.leave_message, None);
    assert_eq!(add_friend.inviter_id, *user1_id);
    let ret = user2
        .lock()
        .await
        .oc()
        .accept_friend(AcceptFriendRequest {
            friend_id: user1_id.into(),
            leave_message: None,
            status: AcceptFriendResult::Success.into(),
        })
        .await
        .unwrap()
        .into_inner();
    let session_id1: SessionID = ret.session_id.unwrap().into();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_some!(
        query_friend(user1_id, user2_id, app.get_db_connection())
            .await
            .unwrap()
    );
    let user1_rec = user1
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(600))
        .await
        .unwrap();
    assert_eq!(user1_rec.len(), 2, "{user1_rec:?}");
    let RespondMsgType::AcceptFriend(accept_friend_notification) =
        user1_rec[1].respond_msg_type.clone().unwrap()
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
    app.async_drop().await;
}

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
    let user2_rec = user2
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(1000))
        .await
        .unwrap();
    assert_eq!(user2_rec.len(), 1);
    let RespondMsgType::AddFriendApproval(add_friend) =
        user2_rec[0].respond_msg_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(add_friend.leave_message, None);
    assert_eq!(add_friend.inviter_id, *user1_id);
    let ret = user2
        .lock()
        .await
        .oc()
        .accept_friend(AcceptFriendRequest {
            friend_id: user1_id.into(),
            leave_message: None,
            status: AcceptFriendResult::Fail.into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_none!(ret.session_id);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_none!(
        query_friend(user1_id, user2_id, app.get_db_connection())
            .await
            .unwrap()
    );
    let user1_rec = user1
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(600))
        .await
        .unwrap();
    assert_eq!(user1_rec.len(), 2, "{user1_rec:?}");
    let RespondMsgType::AcceptFriend(accept_friend_notification) =
        user1_rec[1].respond_msg_type.clone().unwrap()
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
    app.async_drop().await;
}
