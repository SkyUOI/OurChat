use client::http_helper::TestHttpApp;

#[tokio::test]
async fn http_status() {
    let mut app = TestHttpApp::new(None).await.unwrap();
    let response = app
        .http_get("status")
        .await
        .expect("failed")
        .error_for_status()
        .unwrap();
    assert_eq!(response.content_length(), Some(0));
    app.async_drop().await
}
