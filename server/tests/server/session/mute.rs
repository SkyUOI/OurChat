// TODO:check the influence of muted user
use claims::{assert_err, assert_none};
use client::TestApp;
use server::process::error_msg::PERMISSION_DENIED;
use std::time::Duration;

#[tokio::test]
async fn mute_user() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(4, "session1").await.unwrap();
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

    // Test muting single user
    a.lock()
        .await
        .mute_user(vec![bid], session.session_id, None)
        .await
        .unwrap();

    // Test muting multiple users
    a.lock()
        .await
        .mute_user(vec![cid, did], session.session_id, None)
        .await
        .unwrap();

    // Test muting all users
    a.lock()
        .await
        .mute_user(vec![], session.session_id, None)
        .await
        .unwrap();

    app.async_drop().await;
}

#[tokio::test]
async fn mute_user_with_duration() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let (aid, bid) = (a.lock().await.id, b.lock().await.id);

    // Test with 5 seconds duration
    a.lock()
        .await
        .mute_user(
            vec![bid],
            session.session_id,
            Some(std::time::Duration::from_secs(5)),
        )
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(6)).await;
    assert_none!(app.check_mute_status(bid, session.session_id).await);

    app.async_drop().await;
}

#[tokio::test]
async fn mute_user_with_lower_privilege() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    let e = b
        .lock()
        .await
        .mute_user(vec![cid, aid], session.session_id, None)
        .await
        .unwrap_err();

    assert_eq!(e.code(), tonic::Code::PermissionDenied);
    assert_eq!(e.message(), PERMISSION_DENIED);
    app.async_drop().await;
}

#[tokio::test]
async fn mute_already_muted_user() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    a.lock()
        .await
        .mute_user(vec![bid], session.session_id, Some(Duration::from_secs(10)))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(3)).await;

    let e = a
        .lock()
        .await
        .mute_user(
            vec![bid, cid],
            session.session_id,
            Some(Duration::from_secs(10)),
        )
        .await
        .unwrap_err();

    assert_eq!(e.code(), tonic::Code::AlreadyExists);
    app.async_drop().await;
}

#[tokio::test]
async fn unmute_user() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let (aid, bid) = (a.lock().await.id, b.lock().await.id);

    a.lock()
        .await
        .mute_user(vec![bid], session.session_id, None)
        .await
        .unwrap();

    a.lock()
        .await
        .unmute_user(vec![bid], session.session_id)
        .await
        .unwrap();

    assert_none!(app.check_mute_status(bid, session.session_id).await);
    app.async_drop().await;
}
