use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::join_room::v1::JoinRoomRequest;
use server::webrtc::{RoomId, room_members_key};

/// Tests the successful join of a WebRTC room with invitation.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites second user
/// 4. Second user joins the room
/// 5. Verify the response shows success and existing user
/// 6. Check that the member is stored correctly in Redis
/// 7. Verify the user count is updated
#[tokio::test]
async fn join_room_success() {
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

    let room_id = RoomId(create_response.room_id);
    let user2_id = user2.lock().await.id;

    // Admin invites second user
    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *user2_id,
        })
        .await
        .unwrap();

    // Second user joins the room
    let join_request = JoinRoomRequest { room_id: room_id.0 };

    let join_result = user2.lock().await.oc().join_room(join_request).await;
    if join_result.is_err() {
        let err = join_result.unwrap_err();
        panic!("join_room failed: {:?} - {}", err.code(), err.message());
    }
    let join_response = join_result.unwrap().into_inner();

    // Verify response
    assert!(join_response.success, "Join should succeed");
    assert_eq!(
        join_response.existing_users.len(),
        1,
        "Should have one existing user (the creator)"
    );
    assert!(
        join_response
            .existing_users
            .contains(&*user1.lock().await.id),
        "Should contain user1's ID (the creator)"
    );

    // Check Redis for member information
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let member_key = room_members_key(room_id);
    let members: Vec<String> = conn.smembers(&member_key).await.unwrap();

    assert_eq!(
        members.len(),
        2,
        "Should have two members (creator + user2)"
    );
    assert!(members.contains(&user1.lock().await.id.to_string()));
    assert!(members.contains(&user2.lock().await.id.to_string()));

    // Verify user count is updated (creator is counted)
    let room_key_str = server::webrtc::room_key(room_id);
    let room_data = server::webrtc::RoomInfo::from_redis(&mut conn, &room_key_str)
        .await
        .unwrap();

    assert_eq!(
        room_data.users_num, 1,
        "User count should be 1 (only user2 joined after creation)"
    );

    app.async_drop().await;
}

/// Tests joining a room that already has members.
///
/// Steps:
/// 1. Create a test app and three users
/// 2. First user creates a room
/// 3. Admin invites and second user joins
/// 4. Admin invites and third user joins
/// 5. Verify the third user sees existing members
/// 6. Verify all members are stored correctly in Redis
#[tokio::test]
async fn join_room_with_existing_members() {
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

    let room_id = RoomId(create_response.room_id);
    let user2_id = user2.lock().await.id;
    let user3_id = user3.lock().await.id;

    // Admin invites second user
    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *user2_id,
        })
        .await
        .unwrap();

    // Second user joins the room
    user2
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id: *room_id })
        .await
        .unwrap();

    // Admin invites third user
    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *user3_id,
        })
        .await
        .unwrap();

    // Third user joins the room
    let join_response = user3
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id: *room_id })
        .await
        .unwrap()
        .into_inner();

    // Verify response shows existing users
    assert!(join_response.success, "Join should succeed");
    assert_eq!(
        join_response.existing_users.len(),
        2,
        "Should have two existing users (creator + user2)"
    );
    assert!(
        join_response
            .existing_users
            .contains(&*user1.lock().await.id),
        "Should contain user1's ID (the creator)"
    );
    assert!(
        join_response.existing_users.contains(&*user2_id),
        "Should contain user2's ID"
    );

    // Check Redis for all members
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let member_key = room_members_key(room_id);
    let members_count: usize = conn.scard(&member_key).await.unwrap();

    assert_eq!(
        members_count, 3,
        "Should have three members (creator + user2 + user3)"
    );

    app.async_drop().await;
}

/// Tests joining a non-existent room.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Try to join a room that doesn't exist
/// 3. Verify the request fails with not found status
#[tokio::test]
async fn join_room_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Try to join a non-existent room
    let join_request = JoinRoomRequest { room_id: 99999 };

    let result = user.lock().await.oc().join_room(join_request).await;

    // Verify the request failed
    assert!(result.is_err(), "Join should fail for non-existent room");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests that the same user can join a room multiple times (idempotent).
///
/// Steps:
/// 1. Create a test app and user
/// 2. User creates a room
/// 3. User joins the same room twice (after being invited)
/// 4. Verify both joins succeed
/// 5. Verify the user is only counted once
#[tokio::test]
async fn join_room_same_user_multiple_times() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

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
    let user_id = user.lock().await.id;

    // User invites themselves (creator inviting themselves for testing)
    user.lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: *room_id,
            user_id: *user_id,
        })
        .await
        .unwrap();

    // User joins the room first time
    let join_request = JoinRoomRequest { room_id: *room_id };

    user.lock()
        .await
        .oc()
        .join_room(join_request)
        .await
        .unwrap();

    // User joins the room second time (idempotent operation on Redis SADD)
    user.lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id: *room_id })
        .await
        .unwrap();

    // Check Redis - user should only be counted once
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let member_key = room_members_key(room_id);
    let members_count: usize = conn.scard(&member_key).await.unwrap();

    assert_eq!(members_count, 1, "User should only be counted once");

    app.async_drop().await;
}

/// Tests that joining without invitation fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. First user creates a room
/// 3. Second user tries to join without being invited
/// 4. Verify the request fails with PermissionDenied
#[tokio::test]
async fn join_room_not_invited_fails() {
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

    let room_id = RoomId(create_response.room_id);

    // Second user tries to join without being invited
    let join_request = JoinRoomRequest { room_id: room_id.0 };

    let result = user2.lock().await.oc().join_room(join_request).await;

    // Verify the request failed with PermissionDenied
    assert!(result.is_err(), "Join should fail without invitation");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    app.async_drop().await;
}

/// Tests that joining an open_join room succeeds without invitation.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. First user creates an open_join room
/// 3. Second user joins without being invited
/// 4. Verify the request succeeds
#[tokio::test]
async fn join_open_room_without_invitation() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates an open_join room
    let create_request = CreateRoomRequest {
        open_join: true,
        title: Some("Open Room".to_owned()),
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

    // Second user joins without being invited
    let join_request = JoinRoomRequest { room_id };

    let result = user2.lock().await.oc().join_room(join_request).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Join should succeed for open room without invitation"
    );

    let join_response = result.unwrap().into_inner();
    assert!(
        join_response.success,
        "Join response should indicate success"
    );
    assert_eq!(
        join_response.existing_users.len(),
        1,
        "Should see one existing user (the creator)"
    );

    app.async_drop().await;
}

/// Tests that open_join rooms store the open_join flag correctly.
///
/// Steps:
/// 1. Create a test app with user
/// 2. User creates an open_join room
/// 3. Verify room info has open_join set to true
#[tokio::test]
async fn open_join_flag_stored_correctly() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // User creates an open_join room
    let create_request = CreateRoomRequest {
        open_join: true,
        title: Some("Open Room".to_owned()),
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

    let room_id = create_response.room_id;

    // Check Redis for room info
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let room_key_str = server::webrtc::room_key(server::webrtc::RoomId(room_id));
    let room_data = server::webrtc::RoomInfo::from_redis(&mut conn, &room_key_str)
        .await
        .unwrap();

    assert!(
        room_data.open_join,
        "Room should have open_join set to true"
    );

    app.async_drop().await;
}
