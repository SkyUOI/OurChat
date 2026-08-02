use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use server::process::error_msg;

/// Find the first message whose markdown_text matches `text`.
fn find_msg_by_text<'a>(
    msgs: &'a [FetchMsgsResponse],
    text: &str,
) -> Option<&'a pb::service::ourchat::msg_delivery::v1::Msg> {
    msgs.iter()
        .filter_map(|m| match m.respond_event_type.as_ref() {
            Some(RespondEventType::Msg(msg)) if msg.markdown_text == text => Some(msg),
            _ => None,
        })
        .next()
}

#[tokio::test]
async fn test_quote_resolves_quoted_message() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let aid = a.lock().await.id;

    // a sends the original message
    let original = a
        .lock()
        .await
        .send_msg(session.session_id, "hello world", vec![], false)
        .await
        .unwrap()
        .into_inner();
    // b quotes it
    b.lock()
        .await
        .send_msg_with_quote(
            session.session_id,
            "reply to you",
            vec![],
            false,
            original.msg_id,
        )
        .await
        .unwrap();

    let msgs = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    let reply = find_msg_by_text(&msgs, "reply to you").expect("reply message not found in stream");
    assert_eq!(reply.quote_msg_id, original.msg_id);
    assert_eq!(reply.quote_sender_id, u64::from(aid));
    assert_eq!(reply.quote_markdown_text, "hello world");
    assert!(reply.quote_involved_files.is_empty());

    // the original message itself must not carry a quote
    let original_msg =
        find_msg_by_text(&msgs, "hello world").expect("original message not found in stream");
    assert_eq!(original_msg.quote_msg_id, 0);
    app.async_drop().await;
}

#[tokio::test]
async fn test_quote_with_files() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );

    let original = a
        .lock()
        .await
        .send_msg(
            session.session_id,
            "check this file",
            vec!["key1".to_string(), "key2".to_string()],
            false,
        )
        .await
        .unwrap()
        .into_inner();
    b.lock()
        .await
        .send_msg_with_quote(session.session_id, "thanks", vec![], false, original.msg_id)
        .await
        .unwrap();

    let msgs = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    let reply = find_msg_by_text(&msgs, "thanks").expect("reply not found");
    assert_eq!(reply.quote_msg_id, original.msg_id);
    assert_eq!(reply.quote_markdown_text, "check this file");
    assert_eq!(
        reply.quote_involved_files,
        vec!["key1".to_string(), "key2".to_string()]
    );
    app.async_drop().await;
}

#[tokio::test]
async fn test_quote_message_from_other_session_rejected() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user1, session1) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (session_user2, session2) = app
        .new_session_db_level(3, "session2", false)
        .await
        .unwrap();

    let original = session_user1[0]
        .lock()
        .await
        .send_msg(session1.session_id, "in session1", vec![], false)
        .await
        .unwrap()
        .into_inner();

    // a user in session2 (not in session1) tries to quote session1's message
    let err = session_user2[0]
        .lock()
        .await
        .send_msg_with_quote(
            session2.session_id,
            "quote across sessions",
            vec![],
            false,
            original.msg_id,
        )
        .await
        .unwrap_err()
        .unwrap_rpc_status();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::MSG);
    app.async_drop().await;
}

#[tokio::test]
async fn test_quote_nonexistent_message_rejected() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();

    let err = session_user[0]
        .lock()
        .await
        .send_msg_with_quote(
            session.session_id,
            "quote missing msg",
            vec![],
            false,
            u64::MAX,
        )
        .await
        .unwrap_err()
        .unwrap_rpc_status();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), error_msg::not_found::MSG);
    app.async_drop().await;
}

#[tokio::test]
async fn test_quote_encrypted_message_has_no_plaintext() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1", true).await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let aid = a.lock().await.id;

    // a sends an encrypted message (ciphertext stored as markdown_text)
    let original = a
        .lock()
        .await
        .send_msg(session.session_id, "ciphertext", vec![], true)
        .await
        .unwrap()
        .into_inner();
    b.lock()
        .await
        .send_msg_with_quote(
            session.session_id,
            "reply to encrypted",
            vec![],
            false,
            original.msg_id,
        )
        .await
        .unwrap();

    let msgs = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    let reply = find_msg_by_text(&msgs, "reply to encrypted").expect("reply not found");
    assert_eq!(reply.quote_msg_id, original.msg_id);
    assert_eq!(reply.quote_sender_id, u64::from(aid));
    // encrypted quoted messages must not leak plaintext to the server
    assert_eq!(reply.quote_markdown_text, "");
    assert!(reply.quote_involved_files.is_empty());
    app.async_drop().await;
}
