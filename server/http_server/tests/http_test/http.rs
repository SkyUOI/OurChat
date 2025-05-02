use std::path::PathBuf;

use client::http_helper::TestHttpApp;

#[tokio::test]
async fn http_status() {
    tracing::info!("http client building");
    let mut app = TestHttpApp::new(None).await.unwrap();
    tracing::info!("sending request");
    let response = app
        .ourchat_api_get("status")
        .await
        .expect("failed")
        .error_for_status()
        .unwrap();
    tracing::info!("response received");
    assert_eq!(response.content_length(), Some(0));
    app.async_drop().await
}

#[tokio::test]
async fn http_tls() {
    let mut config = TestHttpApp::build_server().await.unwrap();
    config.main_cfg.tls.server_tls_cert_path = Some(PathBuf::from("../test_data/certs/server.pem"));
    config.main_cfg.tls.server_key_cert_path = Some(PathBuf::from("../test_data/certs/server.key"));
    config.main_cfg.tls.client_tls_cert_path = Some(PathBuf::from("../test_data/certs/client.pem"));
    config.main_cfg.tls.client_key_cert_path = Some(PathBuf::from("../test_data/certs/client.key"));
    config.main_cfg.tls.ca_tls_cert_path = Some(PathBuf::from("../test_data/certs/ca.pem"));
    config.main_cfg.tls.client_ca_tls_cert_path =
        Some(PathBuf::from("../test_data/certs/client_ca.pem"));
    config.main_cfg.tls.enable = true;
    let mut http_app = TestHttpApp::setup(config, None, None).await.unwrap();
    let _resp = http_app
        .ourchat_api_get("logo")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    http_app.async_drop().await;
}
