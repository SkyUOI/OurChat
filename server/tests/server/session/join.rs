use base::consts::{ID, SessionID};
use client::TestApp;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use pb::service::ourchat::session::join_in_session::v1::{
    AcceptJoinInSessionRequest, JoinInSessionRequest,
};
use server::db::session::{get_members, in_session};
use std::time::Duration;

#[tokio::test]
async fn join_in_session_success() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    c.lock()
        .await
        .oc()
        .join_in_session(JoinInSessionRequest {
            session_id: session.session_id.into(),
            leave_message: Some("hello".to_string()),
        })
        .await
        .unwrap();
    // will receive
    let join_in_request = a
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(400))
        .await
        .unwrap();
    assert_eq!(join_in_request.len(), 1);
    let RespondMsgType::JoinInSession(join_in) = join_in_request
        .into_iter()
        .next()
        .unwrap()
        .respond_msg_type
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
        .accept_join_in_session(AcceptJoinInSessionRequest {
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
    app.async_drop().await
}

#[tokio::test]
async fn join_in_session_reject() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    c.lock()
        .await
        .oc()
        .join_in_session(JoinInSessionRequest {
            session_id: session.session_id.into(),
            leave_message: Some("hello".to_string()),
        })
        .await
        .unwrap();
    // will receive
    let join_in_request = a
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(400))
        .await
        .unwrap();
    assert_eq!(join_in_request.len(), 1);
    let RespondMsgType::JoinInSession(join_in) = join_in_request
        .into_iter()
        .next()
        .unwrap()
        .respond_msg_type
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
        .accept_join_in_session(AcceptJoinInSessionRequest {
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
    app.async_drop().await
}
