use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::join_room::v1::JoinRoomRequest;
use pb::service::ourchat::webrtc::room::leave_room::v1::LeaveRoomRequest;
use server::webrtc::{RoomId, room_key, room_members_key};

/// Tests the successful leave of a WebRTC room.
///
/// Steps:
/// 1. Create a test app and user
/// 2. User creates a room
/// 3. User invites self and joins the room
/// 4. User leaves the room
/// 5. Verify the response shows success
/// 6. Check that the member is removed from Redis
/// 7. Verify the user count is decremented
#[tokio::test]
async fn leave_room_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let user_id = user.lock().await.id;

    // User creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);

    // User invites self and joins the room
    user.lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: *room_id,
            user_id: *user_id,
        })
        .await
        .unwrap();

    let join_request = JoinRoomRequest { room_id: *room_id };

    user.lock()
        .await
        .oc()
        .join_room(join_request)
        .await
        .unwrap();

    // Verify user is in the room
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let member_key = room_members_key(room_id);
    let members_before: Vec<String> = conn.smembers(&member_key).await.unwrap();
    assert_eq!(members_before.len(), 1);

    // User leaves the room
    let leave_request = LeaveRoomRequest { room_id: *room_id };

    let leave_response = user
        .lock()
        .await
        .oc()
        .leave_room(leave_request)
        .await
        .unwrap()
        .into_inner();

    // Verify response
    assert!(leave_response.success, "Leave should succeed");

    // Check Redis - member should be removed
    let members_after: Vec<String> = conn.smembers(&member_key).await.unwrap();
    assert_eq!(members_after.len(), 0, "Member should be removed");

    // Verify user count is updated
    let room_key_str = room_key(room_id);
    let room_data = server::webrtc::RoomInfo::from_redis(&mut conn, &room_key_str)
        .await
        .unwrap();

    assert_eq!(room_data.users_num, 0, "User count should be 0");

    app.async_drop().await;
}

/// Tests leaving a room that the user hasn't joined.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates and joins a room
/// 3. Second user tries to leave the room (without joining)
/// 4. Verify the request returns success (idempotent operation)
#[tokio::test]
async fn leave_room_not_joined() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user1_id = user1.lock().await.id;

    // First user creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;

    // First user invites self and joins the room
    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id,
            user_id: *user1_id,
        })
        .await
        .unwrap();

    user1
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // Second user tries to leave the room (never joined)
    let leave_response = user2
        .lock()
        .await
        .oc()
        .leave_room(LeaveRoomRequest { room_id })
        .await
        .unwrap()
        .into_inner();

    // Verify response still succeeds (idempotent)
    assert!(
        leave_response.success,
        "Leave should succeed even if not in room"
    );

    app.async_drop().await;
}

/// Tests multiple users joining and leaving a room.
///
/// Steps:
/// 1. Create a test app and three users
/// 2. First user creates a room
/// 3. First user invites all users
/// 4. All three users join the room
/// 5. Verify all members are in Redis
/// 6. Second user leaves
/// 7. Verify member count decreases
/// 8. Third user leaves
/// 9. Verify member count decreases again
#[tokio::test]
async fn leave_room_multiple_users() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();

    // First user creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let room_id_wrapper = RoomId(room_id);
    let member_key = room_members_key(room_id_wrapper);
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;
    let user3_id = user3.lock().await.id;

    // Invite all users
    for user_id in [&user1_id, &user2_id, &user3_id] {
        user1
            .lock()
            .await
            .oc()
            .invite_user_to_room(InviteUserToRoomRequest {
                room_id,
                user_id: **user_id,
            })
            .await
            .unwrap();
    }

    // All users join the room
    user1
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    user2
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    user3
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // Verify all members are in Redis
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let members_count: usize = conn.scard(&member_key).await.unwrap();
    assert_eq!(members_count, 3, "Should have three members");

    // Second user leaves
    user2
        .lock()
        .await
        .oc()
        .leave_room(LeaveRoomRequest { room_id })
        .await
        .unwrap();

    let members_after_second: usize = conn.scard(&member_key).await.unwrap();
    assert_eq!(
        members_after_second, 2,
        "Should have two members after second leaves"
    );

    // Third user leaves
    user3
        .lock()
        .await
        .oc()
        .leave_room(LeaveRoomRequest { room_id })
        .await
        .unwrap();

    let members_after_third: usize = conn.scard(&member_key).await.unwrap();
    assert_eq!(
        members_after_third, 1,
        "Should have one member after third leaves"
    );

    app.async_drop().await;
}

/// Tests leaving a room updates the user count correctly.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. Verify user count is 2
/// 6. One user leaves
/// 7. Verify user count is decremented to 1
#[tokio::test]
async fn leave_room_updates_user_count() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let room_id_wrapper = RoomId(room_id);
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite both users
    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id,
            user_id: *user1_id,
        })
        .await
        .unwrap();

    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id,
            user_id: *user2_id,
        })
        .await
        .unwrap();

    // Both users join the room
    user1
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    user2
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // Check user count is 2
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let room_key_str = room_key(room_id_wrapper);
    let room_data = server::webrtc::RoomInfo::from_redis(&mut conn, &room_key_str)
        .await
        .unwrap();

    assert_eq!(room_data.users_num, 2, "User count should be 2");

    // First user leaves
    user1
        .lock()
        .await
        .oc()
        .leave_room(LeaveRoomRequest { room_id })
        .await
        .unwrap();

    // Verify user count is decremented
    let room_data_after = server::webrtc::RoomInfo::from_redis(&mut conn, &room_key_str)
        .await
        .unwrap();

    assert_eq!(
        room_data_after.users_num, 1,
        "User count should be 1 after leave"
    );

    app.async_drop().await;
}

/// Tests that leaving a non-existent room returns success.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Try to leave a room that doesn't exist
/// 3. Verify the request returns success (graceful handling)
#[tokio::test]
async fn leave_room_non_existent() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Try to leave a non-existent room
    let leave_request = LeaveRoomRequest { room_id: 99999 };

    let leave_response = user
        .lock()
        .await
        .oc()
        .leave_room(leave_request)
        .await
        .unwrap()
        .into_inner();

    // Verify response still succeeds (graceful handling)
    assert!(
        leave_response.success,
        "Leave should succeed even for non-existent room"
    );

    app.async_drop().await;
}
