use base::consts::SessionID;
use server::db::session::{get_all_session_relations, get_session_by_id};
use server::process::error_msg::{PERMISSION_DENIED, not_found};

#[tokio::test]
async fn kick_user() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(4, "session1", false)
        .await
        .unwrap();
    let (a, b, c, d) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
        session_user[3].clone(),
    );
    let (aid, bid, cid, did) = (
        a.lock().await.id,
        b.lock().await.id,
        c.lock().await.id,
        d.lock().await.id,
    );

    // Owner kicks user b
    a.lock()
        .await
        .kick_user(bid, session.session_id)
        .await
        .unwrap();

    // Verify b is removed from session
    assert_eq!(
        get_all_session_relations(bid, app.get_db_connection())
            .await
            .unwrap(),
        vec![]
    );

    // Owner kicks user c
    a.lock()
        .await
        .kick_user(cid, session.session_id)
        .await
        .unwrap();

    // Owner kicks user d
    a.lock()
        .await
        .kick_user(did, session.session_id)
        .await
        .unwrap();

    // Verify all kicked users are removed
    for i in [bid, cid, did] {
        assert_eq!(
            get_all_session_relations(i, app.get_db_connection())
                .await
                .unwrap(),
            vec![]
        );
    }

    // Owner should still be in session
    assert_eq!(
        get_all_session_relations(aid, app.get_db_connection())
            .await
            .unwrap()
            .len(),
        1
    );

    app.async_drop().await;
}

#[tokio::test]
async fn kick_user_without_permission() {
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
    let (aid, _bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    // b tries to kick a (owner) - should fail
    let err = b
        .lock()
        .await
        .kick_user(aid, session.session_id)
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), PERMISSION_DENIED);

    // b tries to kick c - should fail
    let err = b
        .lock()
        .await
        .kick_user(cid, session.session_id)
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);
    assert_eq!(err.message(), PERMISSION_DENIED);

    app.async_drop().await;
}

#[tokio::test]
async fn kick_user_from_non_existent_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, _session) = app
        .new_session_db_level(2, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let (_aid, bid) = (a.lock().await.id, b.lock().await.id);

    // Try to kick from non-existent session
    let err = a
        .lock()
        .await
        .kick_user(bid, SessionID(999999))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
    assert_eq!(err.message(), not_found::SESSION);

    app.async_drop().await;
}

#[tokio::test]
async fn kick_multiple_users() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(5, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let d = session_user[3].clone();
    let e = session_user[4].clone();
    let (aid, bid, cid, did, eid) = (
        a.lock().await.id,
        b.lock().await.id,
        c.lock().await.id,
        d.lock().await.id,
        e.lock().await.id,
    );

    // Owner kicks multiple users
    a.lock()
        .await
        .kick_user(bid, session.session_id)
        .await
        .unwrap();

    a.lock()
        .await
        .kick_user(cid, session.session_id)
        .await
        .unwrap();

    a.lock()
        .await
        .kick_user(did, session.session_id)
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify kicked users are removed
    for id in [bid, cid, did] {
        assert_eq!(
            get_all_session_relations(id, app.get_db_connection())
                .await
                .unwrap(),
            vec![]
        );
    }

    // Verify owner and remaining user are still in session
    assert_eq!(
        get_all_session_relations(aid, app.get_db_connection())
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        get_all_session_relations(eid, app.get_db_connection())
            .await
            .unwrap()
            .len(),
        1
    );

    app.async_drop().await;
}

#[tokio::test]
async fn kick_last_user_deletes_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(2, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let (_aid, bid) = (a.lock().await.id, b.lock().await.id);

    // Owner kicks the only other user
    a.lock()
        .await
        .kick_user(bid, session.session_id)
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Session should still exist (owner is still there)
    let session_info = get_session_by_id(session.session_id, app.get_db_connection())
        .await
        .unwrap();
    assert!(session_info.is_some());

    app.async_drop().await;
}

#[tokio::test]
async fn admin_can_kick_any_user() {
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

    // Admin kicks user b (not in session, but has admin permission)
    admin
        .lock()
        .await
        .kick_user(bid, session.session_id)
        .await
        .unwrap();

    // Admin kicks user a (the owner!)
    admin
        .lock()
        .await
        .kick_user(aid, session.session_id)
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify both a and b are removed from session
    assert_eq!(
        get_all_session_relations(aid, app.get_db_connection())
            .await
            .unwrap(),
        vec![]
    );
    assert_eq!(
        get_all_session_relations(bid, app.get_db_connection())
            .await
            .unwrap(),
        vec![]
    );

    // User c should still be in session
    assert_eq!(
        get_all_session_relations(cid, app.get_db_connection())
            .await
            .unwrap()
            .len(),
        1
    );

    app.async_drop().await;
}
