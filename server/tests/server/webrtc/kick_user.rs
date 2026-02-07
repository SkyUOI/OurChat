use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::accept_room_invitation::v1::AcceptRoomInvitationRequest;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::kick_user::v1::KickUserFromRoomRequest;
use server::webrtc::{RoomId, room_admins_key, room_members_key};

/// Tests that a room admin can kick a user from the room.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. User accepts invitation
/// 4. Admin kicks the user from the room
/// 5. Verify the user is removed from members set in Redis
#[tokio::test]
async fn kick_user_from_room_success() {
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

    // Verify user is in room
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let members_key = room_members_key(room_id);
    let is_member_before: bool = conn.sismember(&members_key, *member_user_id).await.unwrap();
    assert!(is_member_before, "User should be member before kick");

    // Admin kicks user
    let kick_request = KickUserFromRoomRequest {
        room_id: room_id.0,
        user_id: *member_user_id,
    };

    let kick_response = admin_user
        .lock()
        .await
        .oc()
        .kick_user_from_room(kick_request)
        .await
        .unwrap()
        .into_inner();

    assert!(kick_response.success, "Kick should succeed");

    // Check Redis - user should no longer be in members set
    let is_member_after: bool = conn.sismember(&members_key, *member_user_id).await.unwrap();

    assert!(
        !is_member_after,
        "User should not be in members set after kick"
    );

    app.async_drop().await;
}

/// Tests that a non-admin cannot kick users.
///
/// Steps:
/// 1. Create a test app with three users
/// 2. Admin creates a room and invites two users
/// 3. Both users accept invitations
/// 4. First member tries to kick second member (both non-admins)
/// 5. Verify the request fails with PermissionDenied
#[tokio::test]
async fn kick_user_from_room_not_admin() {
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

    // Member1 (non-admin) tries to kick member2
    let kick_request = KickUserFromRoomRequest {
        room_id: room_id.0,
        user_id: *member2_id,
    };

    let result = member1
        .lock()
        .await
        .oc()
        .kick_user_from_room(kick_request)
        .await;

    assert!(result.is_err(), "Non-admin should not be able to kick");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    app.async_drop().await;
}

/// Tests that a user cannot kick themselves.
///
/// Steps:
/// 1. Create a test app with a user
/// 2. User creates a room and invites another user
/// 3. Second user accepts invitation
/// 4. Second user tries to kick themselves
/// 5. Verify the request fails with InvalidArgument
#[tokio::test]
async fn kick_user_from_room_cannot_kick_self() {
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
    admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(
            pb::service::ourchat::webrtc::room::promote_admin::v1::PromoteRoomAdminRequest {
                room_id: room_id.0,
                user_id: *member_user_id,
            },
        )
        .await
        .unwrap();

    // User tries to kick themselves
    let kick_request = KickUserFromRoomRequest {
        room_id: room_id.0,
        user_id: *member_user_id,
    };

    let result = member_user
        .lock()
        .await
        .oc()
        .kick_user_from_room(kick_request)
        .await;

    assert!(
        result.is_err(),
        "User should not be able to kick themselves"
    );

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::InvalidArgument,
        "Should return InvalidArgument status"
    );

    app.async_drop().await;
}

/// Tests that the creator cannot be kicked.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Creator creates a room and invites a user
/// 3. User accepts and is promoted to admin
/// 4. Admin tries to kick the creator
/// 5. Verify the request fails with PermissionDenied
#[tokio::test]
async fn kick_user_from_room_cannot_kick_creator() {
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
        .promote_room_admin(
            pb::service::ourchat::webrtc::room::promote_admin::v1::PromoteRoomAdminRequest {
                room_id: room_id.0,
                user_id: *admin_user_id,
            },
        )
        .await
        .unwrap();

    // Admin tries to kick the creator
    let creator_id = creator_user.lock().await.id;
    let kick_request = KickUserFromRoomRequest {
        room_id: room_id.0,
        user_id: *creator_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .kick_user_from_room(kick_request)
        .await;

    assert!(result.is_err(), "Should not be able to kick creator");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    // Verify creator is still in room
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let members_key = room_members_key(room_id);
    let is_creator_member: bool = conn.sismember(&members_key, *creator_id).await.unwrap();

    assert!(is_creator_member, "Creator should still be member");

    app.async_drop().await;
}

/// Tests that kicking a user who is not in the room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room
/// 3. Admin tries to kick a user who is not in the room
/// 4. Verify the request fails with NotFound
#[tokio::test]
async fn kick_user_from_room_user_not_in_room() {
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

    // Admin tries to kick a user who is not in the room
    let kick_request = KickUserFromRoomRequest {
        room_id: room_id.0,
        user_id: *other_user_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .kick_user_from_room(kick_request)
        .await;

    assert!(result.is_err(), "Kicking non-member should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests that kicking in a non-existent room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Try to kick a user from a non-existent room
/// 3. Verify the request fails with NotFound
#[tokio::test]
async fn kick_user_from_room_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let other_user = app.new_user().await.unwrap();

    let other_user_id = other_user.lock().await.id;

    // Try to kick from a non-existent room
    let kick_request = KickUserFromRoomRequest {
        room_id: 99999,
        user_id: *other_user_id,
    };

    let result = admin_user
        .lock()
        .await
        .oc()
        .kick_user_from_room(kick_request)
        .await;

    assert!(
        result.is_err(),
        "Kicking from non-existent room should fail"
    );

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests that kicking an admin removes them from both members and admins.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. User accepts and is promoted to admin
/// 4. Admin kicks the other admin
/// 5. Verify the user is removed from both members and admins sets
#[tokio::test]
async fn kick_user_from_room_admin_removed_from_both_sets() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let admin2 = app.new_user().await.unwrap();

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
    let admin2_id = admin2.lock().await.id;

    // Admin invites user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *admin2_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    admin2
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // Promote to admin
    admin_user
        .lock()
        .await
        .oc()
        .promote_room_admin(
            pb::service::ourchat::webrtc::room::promote_admin::v1::PromoteRoomAdminRequest {
                room_id: room_id.0,
                user_id: *admin2_id,
            },
        )
        .await
        .unwrap();

    // Verify user is in both sets
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let members_key = room_members_key(room_id);
    let admins_key = room_admins_key(room_id);

    let is_member_before: bool = conn.sismember(&members_key, *admin2_id).await.unwrap();
    let is_admin_before: bool = conn.sismember(&admins_key, *admin2_id).await.unwrap();
    assert!(is_member_before, "User should be member before kick");
    assert!(is_admin_before, "User should be admin before kick");

    // Admin kicks the other admin
    admin_user
        .lock()
        .await
        .oc()
        .kick_user_from_room(KickUserFromRoomRequest {
            room_id: room_id.0,
            user_id: *admin2_id,
        })
        .await
        .unwrap();

    // Check Redis - user should be removed from both sets
    let is_member_after: bool = conn.sismember(&members_key, *admin2_id).await.unwrap();
    let is_admin_after: bool = conn.sismember(&admins_key, *admin2_id).await.unwrap();

    assert!(!is_member_after, "User should not be member after kick");
    assert!(!is_admin_after, "User should not be admin after kick");

    app.async_drop().await;
}
