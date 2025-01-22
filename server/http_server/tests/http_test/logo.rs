use reqwest::header::CONTENT_TYPE;

#[tokio::test]
async fn logo_get() {
    let mut app = client::TestHttpApp::new(None).await.unwrap();
    let resp = app
        .http_get("logo")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), "image/png");
    assert_eq!(
        resp.bytes().await.unwrap().as_ref(),
        include_bytes!("../../../../resource/logo.png")
    );
    app.async_drop().await;
}
