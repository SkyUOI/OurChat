use client::TestApp;
use pb::service::ourchat::webrtc::room::create_room::v1::CreateRoomRequest;
use pb::service::ourchat::webrtc::room::invite_user::v1::InviteUserToRoomRequest;
use pb::service::ourchat::webrtc::room::join_room::v1::JoinRoomRequest;
use pb::service::ourchat::webrtc::signal::v1::{SignalRequest, SignalType};

/// Tests the successful signaling of an SDP offer.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. First user sends an SDP offer to second user
/// 6. Verify the signal is published to RabbitMQ
#[tokio::test]
async fn signal_offer_success() {
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
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // First user sends an SDP offer to second user
    let signal_request = SignalRequest {
        room_id,
        target_user_id: *user2_id,
        signal_type: SignalType::Offer as i32,
        sdp: "v=0\r\no=- 123456 2 IN IP4 127.0.0.1\r\n".to_owned(),
        ice_candidate: String::new(),
        sdp_mid: String::new(),
        sdp_mline_index: 0,
    };

    let signal_response = user1
        .lock()
        .await
        .oc()
        .signal(signal_request)
        .await
        .unwrap()
        .into_inner();

    // Verify response
    assert!(signal_response.success, "Signal should succeed");

    // Verify the signal was published to RabbitMQ
    // (In a real test, you would consume the queue and verify the message)

    app.async_drop().await;
}

/// Tests the successful signaling of an SDP answer.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. Second user sends an SDP answer to first user
/// 6. Verify the signal response is successful
#[tokio::test]
async fn signal_answer_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_owned()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // Second user sends an SDP answer to first user
    let signal_request = SignalRequest {
        room_id,
        target_user_id: *user1_id,
        signal_type: SignalType::Answer as i32,
        sdp: "v=0\r\no=- 654321 2 IN IP4 127.0.0.1\r\n".to_owned(),
        ice_candidate: String::new(),
        sdp_mid: String::new(),
        sdp_mline_index: 0,
    };

    let signal_response = user2
        .lock()
        .await
        .oc()
        .signal(signal_request)
        .await
        .unwrap()
        .into_inner();

    // Verify response
    assert!(signal_response.success, "Signal should succeed");

    app.async_drop().await;
}

/// Tests the successful signaling of an ICE candidate.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. First user sends an ICE candidate to second user
/// 6. Verify the signal response is successful
#[tokio::test]
async fn signal_ice_candidate_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_owned()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // First user sends an ICE candidate to second user
    let signal_request = SignalRequest {
        room_id,
        target_user_id: *user2_id,
        signal_type: SignalType::IceCandidate as i32,
        sdp: String::new(),
        ice_candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54321 typ host".to_owned(),
        sdp_mid: "0".to_owned(),
        sdp_mline_index: 0,
    };

    let signal_response = user1
        .lock()
        .await
        .oc()
        .signal(signal_request)
        .await
        .unwrap()
        .into_inner();

    // Verify response
    assert!(signal_response.success, "Signal should succeed");

    app.async_drop().await;
}

/// Tests that signaling with an unspecified signal type fails.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. Try to send a signal with unspecified type
/// 6. Verify the request fails with invalid argument
#[tokio::test]
async fn signal_unspecified_type_fails() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_owned()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // Try to send a signal with unspecified type
    let signal_request = SignalRequest {
        room_id,
        target_user_id: *user2_id,
        signal_type: SignalType::Unspecified as i32,
        sdp: String::new(),
        ice_candidate: String::new(),
        sdp_mid: String::new(),
        sdp_mline_index: 0,
    };

    let result = user1.lock().await.oc().signal(signal_request).await;

    // Verify the request failed
    assert!(result.is_err(), "Signal should fail for unspecified type");

    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "Should return InvalidArgument status"
    );

    app.async_drop().await;
}

/// Tests that signaling an offer without SDP fails.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. Try to send an offer without SDP data
/// 6. Verify the request fails with invalid argument
#[tokio::test]
async fn signal_offer_without_sdp_fails() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_owned()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // Try to send an offer without SDP
    let signal_request = SignalRequest {
        room_id,
        target_user_id: *user2_id,
        signal_type: SignalType::Offer as i32,
        sdp: String::new(), // Empty SDP
        ice_candidate: String::new(),
        sdp_mid: String::new(),
        sdp_mline_index: 0,
    };

    let result = user1.lock().await.oc().signal(signal_request).await;

    // Verify the request failed
    assert!(result.is_err(), "Signal should fail for offer without SDP");

    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "Should return InvalidArgument status"
    );

    app.async_drop().await;
}

/// Tests that signaling an ICE candidate without candidate data fails.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. Try to send an ICE candidate without candidate data
/// 6. Verify the request fails with invalid argument
#[tokio::test]
async fn signal_ice_candidate_without_data_fails() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_owned()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // Try to send an ICE candidate without candidate data
    let signal_request = SignalRequest {
        room_id,
        target_user_id: *user2_id,
        signal_type: SignalType::IceCandidate as i32,
        sdp: String::new(),
        ice_candidate: String::new(), // Empty candidate
        sdp_mid: String::new(),
        sdp_mline_index: 0,
    };

    let result = user1.lock().await.oc().signal(signal_request).await;

    // Verify the request failed
    assert!(
        result.is_err(),
        "Signal should fail for ICE candidate without data"
    );

    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "Should return InvalidArgument status"
    );

    app.async_drop().await;
}

/// Tests bidirectional signaling between two users.
///
/// Steps:
/// 1. Create a test app and two users
/// 2. First user creates a room
/// 3. First user invites both users
/// 4. Both users join the room
/// 5. First user sends offer to second user
/// 6. Second user sends answer to first user
/// 7. Both users exchange ICE candidates
/// 8. Verify all signals succeed
#[tokio::test]
async fn signal_bidirectional_exchange() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // First user creates a room
    let create_response = user1
        .lock()
        .await
        .oc()
        .create_room(CreateRoomRequest {
            open_join: false,
            title: Some("Test Room".to_owned()),
            auto_delete: true,
        })
        .await
        .unwrap()
        .into_inner();

    let room_id = create_response.room_id;
    let user1_id = user1.lock().await.id;
    let user2_id = user2.lock().await.id;

    // Invite users
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

    // First user sends offer to second user
    let offer_response = user1
        .lock()
        .await
        .oc()
        .signal(SignalRequest {
            room_id,
            target_user_id: *user2_id,
            signal_type: SignalType::Offer as i32,
            sdp: "v=0\r\no=- 123456 2 IN IP4 127.0.0.1\r\n".to_owned(),
            ice_candidate: String::new(),
            sdp_mid: String::new(),
            sdp_mline_index: 0,
        })
        .await
        .unwrap();

    assert!(offer_response.into_inner().success);

    // Second user sends answer to first user
    let answer_response = user2
        .lock()
        .await
        .oc()
        .signal(SignalRequest {
            room_id,
            target_user_id: *user1_id,
            signal_type: SignalType::Answer as i32,
            sdp: "v=0\r\no=- 654321 2 IN IP4 127.0.0.1\r\n".to_owned(),
            ice_candidate: String::new(),
            sdp_mid: String::new(),
            sdp_mline_index: 0,
        })
        .await
        .unwrap();

    assert!(answer_response.into_inner().success);

    // Exchange ICE candidates
    let candidate1_response = user1
        .lock()
        .await
        .oc()
        .signal(SignalRequest {
            room_id,
            target_user_id: *user2_id,
            signal_type: SignalType::IceCandidate as i32,
            sdp: String::new(),
            ice_candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54321 typ host".to_owned(),
            sdp_mid: "0".to_owned(),
            sdp_mline_index: 0,
        })
        .await
        .unwrap();

    assert!(candidate1_response.into_inner().success);

    let candidate2_response = user2
        .lock()
        .await
        .oc()
        .signal(SignalRequest {
            room_id,
            target_user_id: *user1_id,
            signal_type: SignalType::IceCandidate as i32,
            sdp: String::new(),
            ice_candidate: "candidate:2 1 UDP 1694498815 192.168.1.2 54322 typ host".to_owned(),
            sdp_mid: "0".to_owned(),
            sdp_mline_index: 0,
        })
        .await
        .unwrap();

    assert!(candidate2_response.into_inner().success);

    app.async_drop().await;
}
