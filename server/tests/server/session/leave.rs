use pb::service::ourchat::session::leave_session::v1::LeaveSessionRequest;
use server::db::session::{get_all_session_relations, get_session_by_id};

#[tokio::test]
async fn leave_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1", true).await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (aid, _bid, _cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    a.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let session_info = get_session_by_id(session.session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    let relations = get_all_session_relations(aid, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(relations, vec![]);
    assert_eq!(session_info.size, 2);
    app.async_drop().await;
}
