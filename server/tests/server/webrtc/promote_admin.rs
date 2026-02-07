use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::accept_room_invitation::v1::AcceptRoomInvitationRequest;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::promote_admin::v1::PromoteRoomAdminRequest;
use server::webrtc::{RoomId, room_admins_key};

/// Tests that a room admin can promote a member to admin.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. User accepts invitation and joins
/// 4. Admin promotes the user to admin
/// 5. Verify the user is added to admins set in Redis
#[tokio::test]
async fn promote_room_admin_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let member_user = app.new_user().await.unwrap();

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
    let member_user_id = member_user.lock().await.id;

    // Admin invites user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *member_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    member_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Admin promotes user to admin
    let promote_request = PromoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *member_user_id,
    };

    let promote_response = admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(promote_request)
        .await
        .unwrap()
        .into_inner();

    assert!(promote_response.success, "Promotion should succeed");

    // Check Redis - user should be in admins set
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let admins_key = room_admins_key(room_id);
    let is_admin: bool = conn.sismember(&admins_key, *member_user_id).await.unwrap();

    assert!(is_admin, "User should be in admins set");

    app.async_drop().await;
}

/// Tests that a non-admin cannot promote users.
///
/// Steps:
/// 1. Create a test app with three users
/// 2. Admin creates a room and invites two users
/// 3. Both users accept invitations
/// 4. First member tries to promote second member (both non-admins)
/// 5. Verify the request fails with PermissionDenied
#[tokio::test]
async fn promote_room_admin_not_admin() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let member1 = app.new_user().await.unwrap();
    let member2 = app.new_user().await.unwrap();

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
    let member1_id = member1.lock().await.id;
    let member2_id = member2.lock().await.id;

    // Admin invites both users
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *member1_id,
        })
        .await
        .unwrap();

    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *member2_id,
        })
        .await
        .unwrap();

    // Both users accept invitations
    member1
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    member2
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Member1 (non-admin) tries to promote member2
    let promote_request = PromoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *member2_id,
    };

    let result = member1
        .lock()
        .await
        .oc()
        .promote_room_admin(promote_request)
        .await;

    assert!(result.is_err(), "Non-admin should not be able to promote");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    app.async_drop().await;
}

/// Tests that promoting a user not in the room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room
/// 3. Admin tries to promote a user who is not in the room
/// 4. Verify the request fails with NotFound
#[tokio::test]
async fn promote_room_admin_user_not_in_room() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let other_user = app.new_user().await.unwrap();

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
    let other_user_id = other_user.lock().await.id;

    // Admin tries to promote a user who is not in the room
    let promote_request = PromoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *other_user_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(promote_request)
        .await;

    assert!(result.is_err(), "Promoting non-member should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests that promoting to a non-existent room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin tries to promote a user in a non-existent room
/// 3. Verify the request fails with NotFound
#[tokio::test]
async fn promote_room_admin_room_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let other_user = app.new_user().await.unwrap();

    let other_user_id = other_user.lock().await.id;

    // Try to promote in a non-existent room
    let promote_request = PromoteRoomAdminRequest {
        room_id: 99999,
        user_id: *other_user_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(promote_request)
        .await;

    assert!(
        result.is_err(),
        "Promoting in non-existent room should fail"
    );

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests that promoting a user who is already an admin is idempotent.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. User accepts and is promoted to admin
/// 4. Admin tries to promote the same user again
/// 5. Verify the operation succeeds (SADD is idempotent in Redis)
#[tokio::test]
async fn promote_room_admin_already_admin() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let member_user = app.new_user().await.unwrap();

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
    let member_user_id = member_user.lock().await.id;

    // Admin invites user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *member_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    member_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Promote user to admin
    admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(PromoteRoomAdminRequest {
            room_id: room_id.0,
            user_id: *member_user_id,
        })
        .await
        .unwrap();

    // Try to promote again (should succeed due to Redis SADD idempotency)
    let promote_request = PromoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *member_user_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(promote_request)
        .await;

    assert!(result.is_ok(), "Re-promoting an admin should succeed");

    app.async_drop().await;
}
