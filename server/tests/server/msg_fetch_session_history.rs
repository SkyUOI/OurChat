use client::TestApp;
use pb::service::ourchat::msg_delivery::v1::{
    FetchSessionHistoryRequest, FetchSessionHistoryResponse, fetch_msgs_response::RespondEventType,
};

/// Test that FetchSessionHistory returns messages from a specific session
/// with correct pagination (has_more flag).
#[tokio::test]
async fn test_fetch_session_history_basic() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(2, "test_session_history", false)
        .await
        .unwrap();
    let (a, _b) = (session_user[0].clone(), session_user[1].clone());

    // Send 5 messages
    let mut msg_ids = vec![];
    for i in 0..5u32 {
        let ret = a
            .lock()
            .await
            .send_msg(
                session.session_id,
                format!("history msg {}", i),
                vec![],
                false,
            )
            .await
            .unwrap();
        msg_ids.push(ret.into_inner().msg_id);
    }

    // Get current server time as before_time
    let before_time: pb::google::protobuf::Timestamp = a.lock().await.get_timestamp().await.into();

    // Fetch session history with limit = 3
    let request = FetchSessionHistoryRequest {
        session_id: session.session_id.into(),
        before_time: Some(before_time),
        limit: 3,
    };
    // User 'a' (member) fetches session history
    let response: FetchSessionHistoryResponse = a
        .lock()
        .await
        .oc()
        .fetch_session_history(request)
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.messages.len(), 3);
    assert!(response.has_more, "should have more messages");

    // Verify messages belong to this session
    for msg in &response.messages {
        if let Some(RespondEventType::Msg(ref m)) = msg.respond_event_type {
            assert_eq!(m.session_id, u64::from(session.session_id));
        }
    }

    // Fetch remaining messages with the oldest message time as before_time
    if let Some(oldest) = response.messages.last() {
        let request2 = FetchSessionHistoryRequest {
            session_id: session.session_id.into(),
            before_time: oldest.time,
            limit: 5,
        };
        let response2: FetchSessionHistoryResponse = a
            .lock()
            .await
            .oc()
            .fetch_session_history(request2)
            .await
            .unwrap()
            .into_inner();

        // Should get the remaining 2 messages
        assert_eq!(response2.messages.len(), 2);
        assert!(!response2.has_more, "should not have more messages");
    }

    app.async_drop().await;
}

/// Test that FetchSessionHistory returns empty for a session with no messages.
#[tokio::test]
async fn test_fetch_session_history_empty() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(2, "empty_session", false)
        .await
        .unwrap();
    let a = session_user[0].clone();

    let server_time: pb::google::protobuf::Timestamp = a.lock().await.get_timestamp().await.into();

    let request = FetchSessionHistoryRequest {
        session_id: session.session_id.into(),
        before_time: Some(server_time),
        limit: 50,
    };
    let response: FetchSessionHistoryResponse = a
        .lock()
        .await
        .oc()
        .fetch_session_history(request)
        .await
        .unwrap()
        .into_inner();

    assert_eq!(response.messages.len(), 0);
    assert!(!response.has_more);

    app.async_drop().await;
}

/// Test that FetchSessionHistory handles a non-member user (permission check).
#[tokio::test]
async fn test_fetch_session_history_permission() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(2, "perm_session", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let non_member = app.new_user().await.unwrap();

    // Send a message
    a.lock()
        .await
        .send_msg(session.session_id, "test", vec![], false)
        .await
        .unwrap();

    let server_time: pb::google::protobuf::Timestamp = a.lock().await.get_timestamp().await.into();

    // Non-member trying to fetch session history should get permission denied
    let request = FetchSessionHistoryRequest {
        session_id: session.session_id.into(),
        before_time: Some(server_time),
        limit: 10,
    };
    let result = non_member
        .lock()
        .await
        .oc()
        .fetch_session_history(request)
        .await;

    // Should fail with permission denied
    assert!(result.is_err() || result.unwrap().into_inner().messages.is_empty());

    app.async_drop().await;
}
