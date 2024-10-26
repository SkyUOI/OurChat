use crate::helper::TestApp;

#[tokio::test]
async fn test_status() {
    let mut app = TestApp::new(None).await.unwrap();
    let response = app
        .http_get("status")
        .await
        .expect("failed")
        .error_for_status()
        .unwrap();
    assert_eq!(response.content_length(), Some(0));
    app.async_drop().await;
}

#[tokio::test]
async fn test_datetime() {
    let mut app = TestApp::new(None).await.unwrap();
    let _time = app.get_timestamp().await;
    app.async_drop().await;
}
