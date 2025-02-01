use client::TestApp;
use pb::service::ourchat::session::join_in_session::v1::JoinInSessionRequest;

#[tokio::test]
async fn join_in_session() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(2, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = app.new_user().await.unwrap();
    let (aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    let ret = c
        .lock()
        .await
        .oc()
        .join_in_session(JoinInSessionRequest {
            session_id: session.session_id.into(),
            leave_message: Some("hello".to_string()),
        })
        .await
        .unwrap();
    app.async_drop().await
}
