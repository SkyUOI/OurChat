use pb::service::ourchat::session::delete_session::v1::DeleteSessionRequest;
use server::db::session::get_session_by_id;
use server::process::error_msg::{PERMISSION_DENIED, not_found};

#[tokio::test]
async fn delete_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (_aid, _bid, _cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    // delete it without permission
    let err = b
        .lock()
        .await
        .oc()
        .delete_session(DeleteSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), PERMISSION_DENIED);
    // delete an existing session
    a.lock()
        .await
        .oc()
        .delete_session(DeleteSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();
    // delete a non-existing session
    let err = a
        .lock()
        .await
        .oc()
        .delete_session(DeleteSessionRequest { session_id: 0 })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), not_found::SESSION);
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let session_info = get_session_by_id(session.session_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(session_info, None);
    app.async_drop().await;
}
