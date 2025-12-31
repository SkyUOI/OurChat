use pb::service::ourchat::session::delete_session::v1::DeleteSessionRequest;
use server::db::session::get_session_by_id;
use server::process::error_msg::{PERMISSION_DENIED, not_found};

#[tokio::test]
async fn admin_can_delete_any_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    // Create an admin user (not in the session) and promote them
    let admin = app.new_user().await.unwrap();
    admin
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Admin deletes the session (not a member, but has admin permission)
    admin
        .lock()
        .await
        .oc()
        .delete_session(DeleteSessionRequest {
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();

    // Verify all users are removed from the session
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let session_info = get_session_by_id(session.session_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(session_info, None, "Session should be deleted by admin");

    // Verify all users have no session relations
    for user_id in [aid, bid, cid] {
        let relations =
            server::db::session::get_all_session_relations(user_id, app.get_db_connection())
                .await
                .unwrap();
        assert!(
            relations.is_empty(),
            "User {} should have no session relations after admin deletes session",
            user_id
        );
    }

    app.async_drop().await;
}

#[tokio::test]
async fn delete_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
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
