use client::TestApp;
use reqwest::header::CONTENT_TYPE;

async fn check(app: &mut TestApp) {
    tracing::info!("sending request");
    let resp = app
        .ourchat_api_get("logo")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    tracing::info!("response received");
    assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), "image/png");
    assert_eq!(
        resp.bytes().await.unwrap().as_ref(),
        include_bytes!("../../../resource/logo.png")
    );
}

#[tokio::test]
async fn logo_get() {
    tracing::info!("build http client");
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    tracing::info!("check logo");
    check(&mut app).await;
    check(&mut app).await;
    app.async_drop().await;
}
