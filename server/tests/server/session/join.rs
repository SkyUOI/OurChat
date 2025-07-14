use bytes::Bytes;
use client::TestApp;
use client::oc_helper::TestSession;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::service::ourchat::session::allow_user_join_session::v1::AllowUserJoinSessionRequest;
use pb::service::ourchat::session::join_session::v1::JoinSessionRequest;
use rand::rngs::OsRng;
use rsa::pkcs1::DecodeRsaPublicKey as _;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use server::db::session::in_session;

#[tokio::test]
async fn join_in_session_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (_aid, _bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    c.lock()
        .await
        .oc()
        .join_session(JoinSessionRequest {
            session_id: session.session_id.into(),
            leave_message: Some("hello".to_string()),
        })
        .await
        .unwrap();
    // will receive
    let join_request = a.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(join_request.len(), 1);
    let RespondEventType::JoinSessionApproval(join_in) = join_request
        .into_iter()
        .next()
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!()
    };
    // accept
    assert_eq!(join_in.user_id, *cid);
    assert_eq!(join_in.session_id, *session.session_id);
    assert!(
        !in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    assert_eq!(join_in.public_key, Some(c.lock().await.public_key_bytes()));
    let public_key = RsaPublicKey::from_pkcs1_der(&c.lock().await.public_key_bytes()).unwrap();
    let room_key = TestSession::generate_room_key();
    let encrypted_room_key: Bytes = public_key
        .encrypt(&mut OsRng, Pkcs1v15Encrypt, &room_key)
        .unwrap()
        .into();
    a.lock()
        .await
        .oc()
        .allow_user_join_session(AllowUserJoinSessionRequest {
            session_id: session.session_id.into(),
            user_id: join_in.user_id,
            accepted: true,
            room_key: Some(encrypted_room_key),
        })
        .await
        .unwrap();
    assert!(
        in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    let ret = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(ret.len(), 2, "{ret:?}");
    let RespondEventType::AllowUserJoinSessionNotification(ret) =
        ret[1].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    let received_encrypted_room_key = ret.room_key.unwrap();
    let received_room_key: Bytes = c
        .lock()
        .await
        .key_pair
        .0
        .decrypt(Pkcs1v15Encrypt, &received_encrypted_room_key)
        .unwrap()
        .into();
    assert_eq!(received_room_key, room_key);
    assert_eq!(ret.session_id, *session.session_id);
    assert!(ret.accepted);
    app.async_drop().await
}

#[tokio::test]
async fn join_in_session_reject() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(2, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (_aid, _bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    c.lock()
        .await
        .oc()
        .join_session(JoinSessionRequest {
            session_id: session.session_id.into(),
            leave_message: Some("hello".to_string()),
        })
        .await
        .unwrap();
    // will receive
    let join_request = a.lock().await.fetch_msgs().fetch(1).await.unwrap();
    assert_eq!(join_request.len(), 1);
    let RespondEventType::JoinSessionApproval(join_in) = join_request
        .into_iter()
        .next()
        .unwrap()
        .respond_event_type
        .unwrap()
    else {
        panic!()
    };
    // reject
    assert_eq!(join_in.user_id, *cid);
    assert_eq!(join_in.session_id, *session.session_id);
    assert!(
        !in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    a.lock()
        .await
        .oc()
        .allow_user_join_session(AllowUserJoinSessionRequest {
            session_id: session.session_id.into(),
            user_id: join_in.user_id,
            accepted: false,
            room_key: None,
        })
        .await
        .unwrap();
    assert!(
        !in_session(cid, session.session_id, app.get_db_connection())
            .await
            .unwrap()
    );
    let ret = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    assert_eq!(ret.len(), 2, "{ret:?}");
    let RespondEventType::AllowUserJoinSessionNotification(ret) =
        ret[1].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(ret.session_id, *session.session_id);
    assert!(!ret.accepted);
    app.async_drop().await
}
