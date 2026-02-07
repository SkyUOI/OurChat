use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::accept_room_invitation::v1::AcceptRoomInvitationRequest;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use server::webrtc::{RoomId, room_invitations_key, room_members_key};

/// Tests accepting a room invitation successfully.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. Invited user accepts the invitation
/// 4. Verify user is added to members and removed from invitations
#[tokio::test]
async fn accept_room_invitation_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let invited_user = app.new_user().await.unwrap();

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
    let invited_user_id = invited_user.lock().await.id;

    // Admin invites user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *invited_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    let accept_request = AcceptRoomInvitationRequest { room_id: room_id.0 };

    let accept_response = invited_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(accept_request)
        .await
        .unwrap()
        .into_inner();

    assert!(
        accept_response.success,
        "Accepting invitation should succeed"
    );

    // Check Redis - user should be in members, not in invitations
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let members_key = room_members_key(room_id);
    let invitations_key = room_invitations_key(room_id);

    let is_member: bool = conn
        .sismember(&members_key, *invited_user_id)
        .await
        .unwrap();
    let is_invited: bool = conn
        .sismember(&invitations_key, *invited_user_id)
        .await
        .unwrap();

    assert!(is_member, "User should be in members set");
    assert!(!is_invited, "User should not be in invitations set");

    app.async_drop().await;
}

/// Tests that accepting without an invitation fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room (but doesn't invite the user)
/// 3. User tries to accept invitation
/// 4. Verify the request fails with PermissionDenied
#[tokio::test]
async fn accept_room_invitation_not_invited() {
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

    // User tries to accept without being invited
    let accept_request = AcceptRoomInvitationRequest { room_id: room_id.0 };

    let result = other_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(accept_request)
        .await;

    assert!(result.is_err(), "Accepting without invitation should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::PermissionDenied,
        "Should return PermissionDenied status"
    );

    app.async_drop().await;
}

/// Tests that accepting when already in the room fails.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. User accepts invitation and joins the room
/// 4. User tries to accept the invitation again
/// 5. Verify the second acceptance fails with AlreadyExists
#[tokio::test]
async fn accept_room_invitation_already_in_room() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let invited_user = app.new_user().await.unwrap();

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
    let invited_user_id = invited_user.lock().await.id;

    // Admin invites user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *invited_user_id,
        })
        .await
        .unwrap();

    // User accepts invitation
    invited_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // User tries to accept again
    let result = invited_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await;

    assert!(result.is_err(), "Duplicate accept should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::AlreadyExists,
        "Should return AlreadyExists status"
    );

    app.async_drop().await;
}

/// Tests that accepting invitation to a non-existent room fails.
///
/// Steps:
/// 1. Create a test app with a user
/// 2. User tries to accept invitation to a non-existent room
/// 3. Verify the request fails with NotFound
#[tokio::test]
async fn accept_room_invitation_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Try to accept invitation to a non-existent room
    let accept_request = AcceptRoomInvitationRequest { room_id: 99999 };

    let result = user
        .lock()
        .await
        .oc()
        .accept_room_invitation(accept_request)
        .await;

    assert!(result.is_err(), "Accept to non-existent room should fail");

    let status = result.unwrap_err();
    assert!(
        status.code() == tonic::Code::NotFound,
        "Should return NotFound status"
    );

    app.async_drop().await;
}

/// Tests the complete invite-accept workflow.
///
/// Steps:
/// 1. Create a test app with two users
/// 2. Admin creates a room and invites a user
/// 3. Verify user is in invitations set
/// 4. User accepts invitation
/// 5. Verify user is moved from invitations to members
#[tokio::test]
async fn accept_room_invitation_workflow() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin_user = app.new_user().await.unwrap();
    let invited_user = app.new_user().await.unwrap();

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
    let invited_user_id = invited_user.lock().await.id;

    let mut conn = app.get_redis_connection().get().await.unwrap();
    let members_key = room_members_key(room_id);
    let invitations_key = room_invitations_key(room_id);

    // Initially, user should not be in either set
    let is_member: bool = conn
        .sismember(&members_key, *invited_user_id)
        .await
        .unwrap();
    let is_invited: bool = conn
        .sismember(&invitations_key, *invited_user_id)
        .await
        .unwrap();
    assert!(!is_member, "User should not initially be a member");
    assert!(!is_invited, "User should not initially be invited");

    // Admin invites user
    admin_user
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id: room_id.0,
            user_id: *invited_user_id,
        })
        .await
        .unwrap();

    // After invitation, user should be in invitations, not members
    let is_member: bool = conn
        .sismember(&members_key, *invited_user_id)
        .await
        .unwrap();
    let is_invited: bool = conn
        .sismember(&invitations_key, *invited_user_id)
        .await
        .unwrap();
    assert!(!is_member, "User should not be a member after invitation");
    assert!(is_invited, "User should be in invitations after invitation");

    // User accepts invitation
    invited_user
        .lock()
        .await
        .oc()
        .accept_room_invitation(AcceptRoomInvitationRequest { room_id: room_id.0 })
        .await
        .unwrap();

    // After accepting, user should be in members, not invitations
    let is_member: bool = conn
        .sismember(&members_key, *invited_user_id)
        .await
        .unwrap();
    let is_invited: bool = conn
        .sismember(&invitations_key, *invited_user_id)
        .await
        .unwrap();
    assert!(is_member, "User should be a member after accepting");
    assert!(
        !is_invited,
        "User should not be in invitations after accepting"
    );

    app.async_drop().await;
}
