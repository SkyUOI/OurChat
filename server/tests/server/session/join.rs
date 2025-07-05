use client::TestApp;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::service::ourchat::session::join_session::v1::{
    AcceptJoinSessionRequest, JoinSessionRequest,
};
use server::db::session::in_session;

#[tokio::test]
async fn join_in_session_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1").await.unwrap();
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
    a.lock()
        .await
        .oc()
        .accept_join_session(AcceptJoinSessionRequest {
            session_id: session.session_id.into(),
            user_id: join_in.user_id,
            accepted: true,
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
    let RespondEventType::AcceptJoinSession(ret) = ret[1].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(ret.session_id, *session.session_id);
    assert!(ret.accepted);
    app.async_drop().await
}

#[tokio::test]
async fn join_in_session_reject() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1").await.unwrap();
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
        .accept_join_session(AcceptJoinSessionRequest {
            session_id: session.session_id.into(),
            user_id: join_in.user_id,
            accepted: false,
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
    let RespondEventType::AcceptJoinSession(ret) = ret[1].respond_event_type.clone().unwrap()
    else {
        panic!()
    };
    assert_eq!(ret.session_id, *session.session_id);
    assert!(!ret.accepted);
    app.async_drop().await
}
