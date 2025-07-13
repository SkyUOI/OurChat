use claims::{assert_gt, assert_none};
use client::TestApp;
use pb::service::ourchat::session::join_session::v1::JoinSessionRequest;
use server::db::session::{BanStatus, get_all_session_relations};
use server::process::error_msg::{BAN, PERMISSION_DENIED};
use std::time::Duration;

#[tokio::test]
async fn ban_user() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
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

    // Test banning single user
    a.lock()
        .await
        .ban_user(vec![bid], session.session_id, None)
        .await
        .unwrap();

    // Test banning multiple users
    a.lock()
        .await
        .ban_user(vec![cid, did], session.session_id, None)
        .await
        .unwrap();

    // Test banning all users
    a.lock()
        .await
        .ban_user(vec![], session.session_id, None)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    for i in session_user.iter().take(4).skip(1) {
        assert_eq!(
            get_all_session_relations(i.lock().await.id, app.get_db_connection())
                .await
                .unwrap(),
            vec![]
        );
    }
    assert_eq!(
        get_all_session_relations(aid, app.get_db_connection())
            .await
            .unwrap()
            .len(),
        1
    );

    // Try joining the session after being banned
    let result = b
        .lock()
        .await
        .oc()
        .join_session(JoinSessionRequest {
            session_id: session.session_id.into(),
            ..Default::default()
        })
        .await
        .unwrap_err();

    assert_eq!(result.code(), tonic::Code::PermissionDenied);
    assert_eq!(result.message(), BAN);

    app.async_drop().await;
}

#[tokio::test]
async fn ban_user_with_duration() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (_aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);

    // Test with 5 seconds
    a.lock()
        .await
        .ban_user(
            vec![bid, cid],
            session.session_id,
            Some(Duration::from_secs(5)),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_secs(6)).await;
    assert_none!(app.check_ban_status(bid, session.session_id).await.unwrap());
    assert_none!(app.check_ban_status(cid, session.session_id).await.unwrap());
    app.async_drop().await;
}

#[tokio::test]
async fn ban_user_with_lower_privilege() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (aid, _bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    let e = b
        .lock()
        .await
        .ban_user(vec![cid, aid], session.session_id, None)
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied);
    assert_eq!(e.message(), PERMISSION_DENIED);
    app.async_drop().await;
}

#[tokio::test]
async fn ban_already_banned_user() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (_aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    a.lock()
        .await
        .ban_user(vec![bid], session.session_id, Some(Duration::from_secs(10)))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;
    a.lock()
        .await
        .ban_user(
            vec![bid, cid],
            session.session_id,
            Some(Duration::from_secs(10)),
        )
        .await
        .unwrap();
    let BanStatus::RestTime(rest) = app
        .check_ban_status(bid, session.session_id)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!();
    };
    assert_gt!(rest.as_secs(), 10 - 3);
    app.async_drop().await;
}

#[tokio::test]
async fn unban_user() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (_aid, bid, cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    a.lock()
        .await
        .ban_user(vec![bid], session.session_id, None)
        .await
        .unwrap();
    a.lock()
        .await
        .unban_user(vec![bid, cid], session.session_id)
        .await
        .unwrap_err();
    assert_none!(app.check_ban_status(bid, session.session_id).await.unwrap());
    app.async_drop().await;
}
