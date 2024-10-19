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
    let response = app
        .http_get("timestamp")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let text = response.text().await.unwrap();
    println!("{}", text);
    let _time = chrono::DateTime::parse_from_rfc3339(&text).unwrap();
    app.async_drop().await;
}
