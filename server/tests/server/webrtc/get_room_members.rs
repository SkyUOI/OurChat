use client::TestApp;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::get_room_members::v1::GetRoomMembersRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::join_room::v1::JoinRoomRequest;

/// Tests getting members of an empty room.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Create a room
/// 3. User joins their own room (creator is already a member, so join is idempotent)
/// 4. Get room members
/// 5. Verify success and member list contains only the user
#[tokio::test]
async fn get_room_members_empty_room() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let user_id = user.lock().await.id;

    // Create a room
    let create_response = user
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_string()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;

    // Invite self and join (creator is already a member from create_room)
    user.lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id,
            user_id: *user_id,
        })
        .await
        .unwrap();

    user.lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // Get room members (should contain only the user)
    let request = GetRoomMembersRequest { room_id };

    let response = user
        .lock()
        .await
        .oc()
        .get_room_members(request)
        .await
        .unwrap()
        .into_inner();

    assert!(response.success);
    assert_eq!(response.member_count, 1);
    assert_eq!(response.member_ids.len(), 1);
    assert_eq!(response.member_ids[0], user.lock().await.id.0);

    app.async_drop().await;
}

/// Tests getting members of a room with users.
///
/// Steps:
/// 1. Create a test app and users
/// 2. Create a room
/// 3. Owner joins the room
/// 4. Other users are invited and join
/// 5. Get room members
/// 6. Verify member list
#[tokio::test]
async fn get_room_members_with_users() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();

    // Create a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_string()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;
    let user3_id = user3.lock().await.id;

    // Owner joins first (required to call get_room_members)
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

    // Invite and join other users to the room
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

    user2
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    user1
        .lock()
        .await
        .oc()
        .invite_user_to_room(InviteUserToRoomRequest {
            room_id,
            user_id: *user3_id,
        })
        .await
        .unwrap();

    user3
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // Get room members
    let request = GetRoomMembersRequest { room_id };

    let response = user1
        .lock()
        .await
        .oc()
        .get_room_members(request)
        .await
        .unwrap()
        .into_inner();

    assert!(response.success);
    assert_eq!(response.member_count, 3);
    assert_eq!(response.member_ids.len(), 3);

    // Verify all users are in the member list
    let user1_id = user1.lock().await.id.0;
    let user2_id = user2.lock().await.id.0;
    let user3_id = user3.lock().await.id.0;

    assert!(response.member_ids.contains(&user1_id));
    assert!(response.member_ids.contains(&user2_id));
    assert!(response.member_ids.contains(&user3_id));

    app.async_drop().await;
}

/// Tests getting members of a non-existent room.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Try to get members of non-existent room
/// 3. Verify not found error
#[tokio::test]
async fn get_room_members_not_found() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Try to get members of non-existent room
    let request = GetRoomMembersRequest { room_id: 999 };

    let result = user.lock().await.oc().get_room_members(request).await;

    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);

    app.async_drop().await;
}

/// Tests that non-members cannot get room members.
///
/// Steps:
/// 1. Create a test app and users
/// 2. Create a room
/// 3. Invite and user2 joins, User3 doesn't
/// 4. User3 tries to get members
/// 5. Verify permission denied
#[tokio::test]
async fn get_room_members_not_in_room() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();

    // Create a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_string()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user2_id = user2.lock().await.id;

    // Only user2 joins (after invitation)
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

    user2
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // User3 tries to get members without joining
    let result = user3
        .lock()
        .await
        .oc()
        .get_room_members(GetRoomMembersRequest { room_id })
        .await;

    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::PermissionDenied);

    app.async_drop().await;
}

/// Tests getting members after user leaves.
///
/// Steps:
/// 1. Create a test app and users
/// 2. Create a room
/// 3. Owner joins
/// 4. Invite and user2 joins
/// 5. Verify 2 members
/// 6. User2 leaves
/// 7. Verify 1 member
#[tokio::test]
async fn get_room_members_after_leave() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // Create a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_string()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Owner joins first
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

    // User2 joins (after invitation)
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

    user2
        .lock()
        .await
        .oc()
        .join_room(JoinRoomRequest { room_id })
        .await
        .unwrap();

    // Verify 2 members
    let response = user1
        .lock()
        .await
        .oc()
        .get_room_members(GetRoomMembersRequest { room_id })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.member_count, 2);

    // User2 leaves
    use pb::service::ourchat::webrtc::room::leave_room::v1::LeaveRoomRequest;
    user2
        .lock()
        .await
        .oc()
        .leave_room(LeaveRoomRequest { room_id })
        .await
        .unwrap();

    // Verify 1 member
    let response = user1
        .lock()
        .await
        .oc()
        .get_room_members(GetRoomMembersRequest { room_id })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.member_count, 1);
    assert_eq!(response.member_ids[0], user1.lock().await.id.0);

    app.async_drop().await;
}
