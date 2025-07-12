use std::time::Duration;

use client::TestApp;
use migration::MigratorTrait;
use server::db::session::in_session;

#[tokio::test]
async fn db_migration() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let db_name = app.db_url.clone();
    app.should_drop_db = false;
    app.async_drop().await;
    let db = sea_orm::Database::connect(&db_name).await.unwrap();
    migration::Migrator::down(&db, None).await.unwrap()
}

#[tokio::test]
async fn test_in_session() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let a = session_user[0].clone();
    let b = session_user[1].clone();
    let c = session_user[2].clone();
    let (aid, _bid, _cid) = (a.lock().await.id, b.lock().await.id, c.lock().await.id);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        in_session(aid, session.session_id, app.get_db_connection())
            .await
            .unwrap(),
        "{aid} is not in session {session_id}",
        session_id = session.session_id
    );
    app.async_drop().await;
}
