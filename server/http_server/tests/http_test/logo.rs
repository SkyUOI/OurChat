use client::TestHttpApp;
use reqwest::header::CONTENT_TYPE;

async fn check(app: &mut TestHttpApp) {
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
}

#[tokio::test]
async fn logo_get() {
    let mut app = TestHttpApp::new(None).await.unwrap();
    check(&mut app).await;
    check(&mut app).await;
    app.async_drop().await;
}
