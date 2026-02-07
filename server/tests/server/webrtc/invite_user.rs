use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use server::webrtc::{RoomId, room_invitations_key};

/// Tests that a room admin can invite a user to a room.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. First user creates a room (becomes admin)
/// 3. Admin invites second user to the room
/// 4. Verify the invitation is stored in Redis
#[tokio::test]
async fn invite_user_to_room_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    // Admin creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = admin_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let target_user_id = target_user.lock().await.id;

    // Admin invites target user
    let invite_request = InviteUserToRoomRequest {
        room_id: room_id.0,
        user_id: *target_user_id,
    };

    let invite_response = admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(invite_request)
        .await
        .unwrap()
        .into_inner();

    assert!(invite_response.success, "Invitation should succeed");

    // Check Redis for invitation
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let invitations_key = room_invitations_key(room_id);
    let is_invited: bool = conn
        .sismember(&invitations_key, *target_user_id)
        .await
        .unwrap();

    assert!(is_invited, "User should be in invitations set");

    app.async_drop().await;
}

/// Tests that a non-admin cannot invite users to a room.
///
/// Steps:
/// 1. Create a test app with three users
/// 2. First user creates a room
/// 3. Second user joins the room
/// 4. Second user (non-admin) tries to invite third user
/// 5. Verify the request fails with permission denied
#[tokio::test]
async fn invite_user_to_room_not_admin() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let regular_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    // Admin creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = admin_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let regular_user_id = regular_user.lock().await.id;

    // Admin invites regular user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *regular_user_id,
        })
        .await
        .unwrap();

    // Regular user joins the room
    regular_user
        .lock()
        .await
        .oc()
        .join_room(
            pb::service::ourchat::webrtc::room::join_room::v1::JoinRoomRequest {
                room_id: room_id.0,
            },
        )
        .await
        .unwrap();

    let target_user_id = target_user.lock().await.id;

    // Non-admin tries to invite a user
    let invite_request = InviteUserToRoomRequest {
        room_id: room_id.0,
        user_id: *target_user_id,
    };

    let result = regular_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(invite_request)
        .await;

    assert!(result.is_err(), "Non-admin should not be able to invite");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    app.async_drop().await;
}

/// Tests that inviting a user twice returns an error.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. First user creates a room
/// 3. Admin invites the same user twice
/// 4. Verify the second invitation fails with AlreadyExists
#[tokio::test]
async fn invite_user_to_room_already_invited() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    // Admin creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = admin_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let target_user_id = target_user.lock().await.id;

    // Admin invites target user first time
    let invite_request = InviteUserToRoomRequest {
        room_id: room_id.0,
        user_id: *target_user_id,
    };

    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(invite_request)
        .await
        .unwrap();

    // Try to invite the same user again
    let result = admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(invite_request)
        .await;

    assert!(result.is_err(), "Duplicate invitation should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::AlreadyExists,
        "Should return AlreadyExists status"
    );

    app.async_drop().await;
}

/// Tests that inviting to a non-existent room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Try to invite a user to a non-existent room
/// 3. Verify the request fails with NotFound
#[tokio::test]
async fn invite_user_to_room_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    let target_user_id = target_user.lock().await.id;

    // Try to invite to a non-existent room
    let invite_request = InviteUserToRoomRequest {
        room_id: 99999,
        user_id: *target_user_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(invite_request)
        .await;

    assert!(
        result.is_err(),
        "Invitation to non-existent room should fail"
    );

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}
