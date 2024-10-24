use crate::helper;

#[tokio::test]
async fn test_text_sent() {
    let mut test_app = helper::TestApp::new(None).await.unwrap();
    let group = test_app.new_session(3, "group1").await.unwrap();
    test_app.async_drop().await;
}
