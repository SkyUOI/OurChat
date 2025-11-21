use pb::service::ourchat::session::leave_session::v1::LeaveSessionRequest;
use server::db::session::{get_all_session_relations, get_session_by_id, get_members};
use entities::user_role_relation;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};

/// Test that when the last user leaves a session, the session is automatically deleted
#[tokio::test]
async fn last_user_leaves_session_deletes_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Create a session with only one user
    let (session_user, session) = app.new_session_db_level(1, "single_user_session", false).await.unwrap();
    let user = session_user[0].clone();
    let user_id = user.lock().await.id;
    let session_id = session.session_id;

    // Verify session exists and has correct size
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 1);

    // User leaves session (should trigger automatic deletion)
    user.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session_id.into(),
        })
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify session no longer exists
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(session_info, None);

    // Verify user has no session relations
    let relations = get_all_session_relations(user_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(relations, vec![]);

    // Verify no role relations exist for this session
    let role_relations: Vec<entities::user_role_relation::Model> = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(session_id))
        .all(app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(role_relations, vec![]);

    app.async_drop().await;
}

/// Test that when multiple users are in a session, leaving doesn't delete the session
#[tokio::test]
async fn multiple_users_leaving_does_not_delete_session() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Create a session with multiple users
    let (session_user, session) = app.new_session_db_level(3, "multi_user_session", false).await.unwrap();
    let user_a = session_user[0].clone();
    let user_b = session_user[1].clone();
    let user_c = session_user[2].clone();
    let (user_a_id, user_b_id, user_c_id) = (
        user_a.lock().await.id,
        user_b.lock().await.id,
        user_c.lock().await.id,
    );
    let session_id = session.session_id;

    // Verify initial session state
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 3);

    // User A leaves session
    user_a.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session_id.into(),
        })
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify session still exists but size decreased
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 2);

    // Verify User A has no session relations
    let user_a_relations = get_all_session_relations(user_a_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_a_relations, vec![]);

    // Verify User B and C still have session relations
    let user_b_relations = get_all_session_relations(user_b_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_b_relations.len(), 1);

    let user_c_relations = get_all_session_relations(user_c_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_c_relations.len(), 1);

    // User B leaves session
    user_b.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session_id.into(),
        })
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify session still exists with size 1
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 1);

    // Verify User B has no session relations
    let user_b_relations = get_all_session_relations(user_b_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_b_relations, vec![]);

    // Verify User C still has session relation
    let user_c_relations = get_all_session_relations(user_c_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_c_relations.len(), 1);

    app.async_drop().await;
}

/// Test that when the last user leaves, all session data is properly cleaned up
#[tokio::test]
async fn last_user_leaves_cleans_up_all_session_data() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Create a session with multiple users
    let (session_user, session) = app.new_session_db_level(2, "cleanup_test_session", false).await.unwrap();
    let user_a = session_user[0].clone();
    let user_b = session_user[1].clone();
    let (user_a_id, user_b_id) = (user_a.lock().await.id, user_b.lock().await.id);
    let session_id = session.session_id;

    // Verify initial state
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 2);

    let members = get_members(session_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(members.len(), 2);

    let role_relations: Vec<entities::user_role_relation::Model> = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(session_id))
        .all(app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(role_relations.len(), 2);

    // User A leaves (not the last user)
    user_a.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session_id.into(),
        })
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify session still exists
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 1);

    // User B leaves (last user - should trigger cleanup)
    user_b.lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session_id.into(),
        })
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Verify session no longer exists
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(session_info, None);

    // Verify no session relations exist
    let all_session_relations = get_members(session_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(all_session_relations, vec![]);

    // Verify no role relations exist
    let role_relations: Vec<entities::user_role_relation::Model> = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::SessionId.eq(session_id))
        .all(app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(role_relations, vec![]);

    // Verify users have no session relations
    let user_a_relations = get_all_session_relations(user_a_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_a_relations, vec![]);

    let user_b_relations = get_all_session_relations(user_b_id, app.get_db_connection())
        .await
        .unwrap();
    assert_eq!(user_b_relations, vec![]);

    app.async_drop().await;
}

/// Test edge case: leaving a session that doesn't exist
#[tokio::test]
async fn leave_nonexistent_session_returns_error() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    let user = app.new_user().await.unwrap();

    // Try to leave a session that doesn't exist
    let err = user
        .lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: 999999, // Non-existent session ID
        })
        .await
        .unwrap_err();

    // Should get a "not found" error
    assert_eq!(err.code(), tonic::Code::NotFound);

    app.async_drop().await;
}

/// Test edge case: user not in session tries to leave
#[tokio::test]
async fn user_not_in_session_cannot_leave() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Create a session with one user
    let (session_user, session) = app.new_session_db_level(1, "single_user_session", false).await.unwrap();
    let session_id = session.session_id;

    // Create a different user not in the session
    let user_not_in_session = app.new_user().await.unwrap();

    // User not in session tries to leave
    let err = user_not_in_session
        .lock()
        .await
        .oc()
        .leave_session(LeaveSessionRequest {
            session_id: session_id.into(),
        })
        .await
        .unwrap_err();

    // Should get a "not found" error (user not in session)
    assert_eq!(err.code(), tonic::Code::NotFound);

    // Verify session still exists
    let session_info = get_session_by_id(session_id, app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session_info.size, 1);

    app.async_drop().await;
}