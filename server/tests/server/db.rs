use client::TestApp;
use migration::MigratorTrait;

#[tokio::test]
async fn db_migration() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let db_name = app.db_url.clone();
    app.should_drop_db = false;
    app.async_drop().await;
    let db = sea_orm::Database::connect(&db_name).await.unwrap();
    migration::Migrator::down(&db, None).await.unwrap()
}
