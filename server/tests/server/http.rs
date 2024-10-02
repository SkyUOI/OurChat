use crate::helper::TestApp;

#[tokio::test]
async fn test_status() {
    let mut app = TestApp::new_logined(None).await.unwrap();
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/v1/status", app.http_port))
        .send()
        .await
        .expect("failed");
    assert!(response.status().is_success(), "{:?}", response.status());
    assert_eq!(response.content_length(), Some(0));
    app.async_drop().await;
}
