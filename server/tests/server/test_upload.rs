use crate::helper;

#[tokio::test]
async fn test_upload() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let _ = app.new_user_logined().await.unwrap();
    app.async_drop().await;
}
