use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::accept_room_invitation::v1::AcceptRoomInvitationRequest;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::demote_admin::v1::DemoteRoomAdminRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::promote_admin::v1::PromoteRoomAdminRequest;
use server::webrtc::{RoomId, room_admins_key};

/// Tests that a room creator can demote an admin.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Creator creates a room and invites a user
/// 3. User accepts invitation
/// 4. Creator promotes user to admin
/// 5. Creator demotes the user back to member
/// 6. Verify the user is removed from admins set in Redis
#[tokio::test]
async fn demote_room_admin_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let creator_user = app.new_user().await.unwrap();
    let admin_user = app.new_user().await.unwrap();

    // Creator creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = creator_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let admin_user_id = admin_user.lock().await.id;

    // Creator invites user
    creator_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *admin_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    admin_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Creator promotes user to admin
    creator_user
        .lock()
        .await
        .oc()
        .promote_room_admin(PromoteRoomAdminRequest {
            room_id: room_id.0,
            user_id: *admin_user_id,
        })
        .await
        .unwrap();

    // Verify user is admin
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let admins_key = room_admins_key(room_id);
    let is_admin_before: bool = conn.sismember(&admins_key, *admin_user_id).await.unwrap();
    assert!(is_admin_before, "User should be admin before demotion");

    // Creator demotes user
    let demote_request = DemoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *admin_user_id,
    };

    let demote_response = creator_user
        .lock()
        .await
        .oc()
        .demote_room_admin(demote_request)
        .await
        .unwrap()
        .into_inner();

    assert!(demote_response.success, "Demotion should succeed");

    // Check Redis - user should no longer be in admins set
    let is_admin_after: bool = conn.sismember(&admins_key, *admin_user_id).await.unwrap();

    assert!(
        !is_admin_after,
        "User should not be in admins set after demotion"
    );

    app.async_drop().await;
}

/// Tests that a non-creator cannot demote admins.
///
/// Steps:
/// 1. Create a test app with three users
/// 2. Creator creates a room and invites two users
/// 3. Both users accept and are promoted to admins
/// 4. First admin tries to demote second admin
/// 5. Verify the request fails with PermissionDenied
#[tokio::test]
async fn demote_room_admin_not_creator() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let creator_user = app.new_user().await.unwrap();
    let admin1 = app.new_user().await.unwrap();
    let admin2 = app.new_user().await.unwrap();

    // Creator creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = creator_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let admin1_id = admin1.lock().await.id;
    let admin2_id = admin2.lock().await.id;

    // Creator invites both users
    creator_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *admin1_id,
        })
        .await
        .unwrap();

    creator_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *admin2_id,
        })
        .await
        .unwrap();

    // Both users accept invitations
    admin1
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    admin2
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Creator promotes both to admins
    creator_user
        .lock()
        .await
        .oc()
        .promote_room_admin(PromoteRoomAdminRequest {
            room_id: room_id.0,
            user_id: *admin1_id,
        })
        .await
        .unwrap();

    creator_user
        .lock()
        .await
        .oc()
        .promote_room_admin(PromoteRoomAdminRequest {
            room_id: room_id.0,
            user_id: *admin2_id,
        })
        .await
        .unwrap();

    // Admin1 (non-creator) tries to demote admin2
    let demote_request = DemoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *admin2_id,
    };

    let result = admin1
        .lock()
        .await
        .oc()
        .demote_room_admin(demote_request)
        .await;

    assert!(result.is_err(), "Non-creator should not be able to demote");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    app.async_drop().await;
}

/// Tests that the creator cannot be demoted.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Creator creates a room and invites a user
/// 3. User accepts and is promoted to admin
/// 4. Admin tries to demote the creator
/// 5. Verify the request fails with PermissionDenied
#[tokio::test]
async fn demote_room_admin_cannot_demote_creator() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let creator_user = app.new_user().await.unwrap();
    let admin_user = app.new_user().await.unwrap();

    // Creator creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = creator_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let admin_user_id = admin_user.lock().await.id;

    // Creator invites user
    creator_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *admin_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    admin_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Creator promotes user to admin
    creator_user
        .lock()
        .await
        .oc()
        .promote_room_admin(PromoteRoomAdminRequest {
            room_id: room_id.0,
            user_id: *admin_user_id,
        })
        .await
        .unwrap();

    // Admin tries to demote the creator
    let creator_id = creator_user.lock().await.id;
    let demote_request = DemoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *creator_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .demote_room_admin(demote_request)
        .await;

    assert!(result.is_err(), "Should not be able to demote creator");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    // Verify creator is still admin
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let admins_key = room_admins_key(room_id);
    let is_creator_admin: bool = conn.sismember(&admins_key, *creator_id).await.unwrap();

    assert!(is_creator_admin, "Creator should still be admin");

    app.async_drop().await;
}

/// Tests that demoting in a non-existent room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Try to demote a user in a non-existent room
/// 3. Verify the request fails with NotFound
#[tokio::test]
async fn demote_room_admin_room_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let creator_user = app.new_user().await.unwrap();
    let other_user = app.new_user().await.unwrap();

    let other_user_id = other_user.lock().await.id;

    // Try to demote in a non-existent room
    let demote_request = DemoteRoomAdminRequest {
        room_id: 99999,
        user_id: *other_user_id,
    };

    let result = creator_user
        .lock()
        .await
        .oc()
        .demote_room_admin(demote_request)
        .await;

    assert!(result.is_err(), "Demoting in non-existent room should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests that demoting a user who is not an admin is handled gracefully.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Creator creates a room and invites a user
/// 3. User accepts invitation (as regular member)
/// 4. Creator tries to demote the user (who is not an admin)
/// 5. Verify the operation succeeds (SREM is idempotent in Redis)
#[tokio::test]
async fn demote_room_admin_not_an_admin() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let creator_user = app.new_user().await.unwrap();
    let member_user = app.new_user().await.unwrap();

    // Creator creates a room
    let create_request = CreateRoomRequest {
        open_join: false,
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    let create_response = creator_user
        .lock()
        .await
        .oc()
        .create_room(create_request)
        .await
        .unwrap()
        .into_inner();

    let room_id = RoomId(create_response.room_id);
    let member_user_id = member_user.lock().await.id;

    // Creator invites user
    creator_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *member_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation (but is not promoted)
    member_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Creator tries to demote the user (who is not an admin)
    let demote_request = DemoteRoomAdminRequest {
        room_id: room_id.0,
        user_id: *member_user_id,
    };

    let result = creator_user
        .lock()
        .await
        .oc()
        .demote_room_admin(demote_request)
        .await;

    // Should succeed due to Redis SREM idempotency
    assert!(
        result.is_ok(),
        "Demoting non-admin should succeed (idempotent)"
    );

    app.async_drop().await;
}
