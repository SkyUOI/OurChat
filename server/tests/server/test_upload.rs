use crate::helper;

#[tokio::test]
async fn test_upload() {
    let mut app = helper::TestApp::new_logined(None).await.unwrap();
    app.async_drop().await;
}
