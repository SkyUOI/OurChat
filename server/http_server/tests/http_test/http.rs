use client::http_helper::TestHttpApp;

#[tokio::test]
async fn http_status() {
    tracing::info!("http client building");
    let mut app = TestHttpApp::new(None).await.unwrap();
    tracing::info!("sending request");
    let response = app
        .http_get("status")
        .await
        .expect("failed")
        .error_for_status()
        .unwrap();
    tracing::info!("response received");
    assert_eq!(response.content_length(), Some(0));
    app.async_drop().await
}
