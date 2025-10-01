use client::TestApp;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use server::webrtc::{RoomId, RoomInfo, empty_room_name, room_key};

/// Tests the successful creation of a WebRTC room with a title and auto_delete enabled.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Call create_room with a title and auto_delete set to true
/// 3. Verify the response contains a valid room_id
/// 4. Check that the room information is stored correctly in Redis
/// 5. Verify the room is added to the empty rooms set
#[tokio::test]
async fn create_room_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Create room request
    let request = CreateRoomRequest {
        title: Some("Test Room".to_owned()),
        auto_delete: true,
    };

    // Call create_room
    let response = user
        .lock()
        .await
        .oc()
        .create_room(request)
        .await
        .unwrap()
        .into_inner();

    // Verify response contains room_id
    assert!(response.room_id > 0, "Room ID should be positive");

    let room_id = RoomId(response.room_id);

    // Check Redis for room information
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let key = room_key(room_id);

    // Verify room info is stored correctly
    let stored_data = RoomInfo::from_redis(&mut conn, &key).await.unwrap();

    assert_eq!(stored_data.title, Some("Test Room".to_owned()));
    assert_eq!(stored_data.room_id, room_id);
    assert_eq!(stored_data.users_num, 0);
    assert!(stored_data.auto_delete);

    // Verify room is in empty rooms set
    let is_in_empty_set: bool = conn.sismember(empty_room_name(), room_id).await.unwrap();
    assert!(is_in_empty_set, "Room should be in empty rooms set");

    app.async_drop().await;
}

/// Tests the successful creation of a WebRTC room with no title and auto_delete disabled.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Call create_room with no title and auto_delete set to false
/// 3. Verify the response contains a valid room_id
/// 4. Check that the room information is stored correctly in Redis
/// 5. Verify the room is added to the empty rooms set
#[tokio::test]
async fn create_room_no_title() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Create room request without title
    let request = CreateRoomRequest {
        title: None,
        auto_delete: false,
    };

    // Call create_room
    let response = user
        .lock()
        .await
        .oc()
        .create_room(request)
        .await
        .unwrap()
        .into_inner();

    // Verify response contains room_id
    assert!(response.room_id > 0, "Room ID should be positive");

    let room_id = RoomId(response.room_id);

    // Check Redis for room information
    let mut conn = app.get_redis_connection().get().await.unwrap();
    let key = room_key(room_id);

    // Verify room info is stored correctly
    let stored_data = RoomInfo::from_redis(&mut conn, &key).await.unwrap();

    assert_eq!(stored_data.title, None);
    assert_eq!(stored_data.room_id, room_id);
    assert_eq!(stored_data.users_num, 0);
    assert!(!stored_data.auto_delete);

    // Verify room is in empty rooms set
    let is_in_empty_set: bool = conn.sismember(empty_room_name(), room_id).await.unwrap();
    assert!(is_in_empty_set, "Room should be in empty rooms set");

    app.async_drop().await;
}

/// Tests that multiple room creations generate unique room IDs.
///
/// Steps:
/// 1. Create a test app and user
/// 2. Create multiple rooms
/// 3. Verify all room IDs are unique
#[tokio::test]
async fn create_multiple_rooms_unique_ids() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let mut room_ids = std::collections::HashSet::new();

    // Create multiple rooms
    for i in 0..3 {
        let request = CreateRoomRequest {
            title: Some(format!("Room {}", i)),
            auto_delete: i % 2 == 0,
        };

        let response = user
            .lock()
            .await
            .oc()
            .create_room(request)
            .await
            .unwrap()
            .into_inner();

        // Verify room ID is unique
        assert!(
            room_ids.insert(response.room_id),
            "Room ID should be unique: {}",
            response.room_id
        );
    }

    app.async_drop().await;
}
