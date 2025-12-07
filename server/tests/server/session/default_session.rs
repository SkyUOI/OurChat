use base::consts::SessionID;
use client::TestApp;
use sea_orm::TransactionTrait;
use server::db::session::{get_session_by_id, in_session, join_in_session_or_create};

#[tokio::test]
async fn test_join_in_session_or_create() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let user_id = user.lock().await.id;
    let session_id = SessionID(999999);

    // First call: session does not exist, should create session
    let transaction = app.get_db_connection().begin().await.unwrap();
    join_in_session_or_create(session_id, user_id, None, &transaction, false)
        .await
        .unwrap();
    transaction.commit().await.unwrap();

    // Verify session exists and user is a member
    let session = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .expect("session should exist");
    assert_eq!(session.session_id, session_id.0 as i64);
    assert_eq!(session.size, 1); // one user added
    assert!(!session.e2ee_on); // e2ee_on = false
    assert!(
        in_session(user_id, session_id, app.get_db_connection())
            .await
            .unwrap()
    );

    // Second call: session already exists, should just join (but user already in session)
    // This should fail due to duplicate entry (unique constraint)
    let transaction = app.get_db_connection().begin().await.unwrap();
    let result = join_in_session_or_create(session_id, user_id, None, &transaction, true).await;
    // Expect error because user already in session (unique constraint violation)
    assert!(result.is_err());
    transaction.rollback().await.unwrap();

    // Test with a different user, should join existing session
    let user2 = app.new_user().await.unwrap();
    let user2_id = user2.lock().await.id;
    let transaction = app.get_db_connection().begin().await.unwrap();
    join_in_session_or_create(session_id, user2_id, None, &transaction, false)
        .await
        .unwrap();
    transaction.commit().await.unwrap();

    let session = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .expect("session should exist");
    assert_eq!(session.size, 2); // now two users
    assert!(
        in_session(user2_id, session_id, app.get_db_connection())
            .await
            .unwrap()
    );

    // Test with e2ee_on = true creating new session
    let session_id2 = SessionID(888888);
    let transaction = app.get_db_connection().begin().await.unwrap();
    join_in_session_or_create(session_id2, user_id, None, &transaction, true)
        .await
        .unwrap();
    transaction.commit().await.unwrap();

    let session2 = get_session_by_id(session_id2, app.get_db_connection())
        .await
        .unwrap()
        .expect("session should exist");
    assert!(session2.e2ee_on); // e2ee_on = true

    app.async_drop().await;
}
