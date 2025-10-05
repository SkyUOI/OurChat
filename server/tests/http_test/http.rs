use std::path::PathBuf;

use client::TestApp;
use http::StatusCode;

#[tokio::test]
async fn http_status() {
    tracing::info!("http client building");
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
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
    let mut config = TestApp::get_test_config().unwrap();
    config.0.http_cfg.tls.server_tls_cert_path = Some(PathBuf::from("test_data/certs/server.pem"));
    config.0.http_cfg.tls.server_key_cert_path = Some(PathBuf::from("test_data/certs/server.key"));
    config.0.http_cfg.tls.client_tls_cert_path = Some(PathBuf::from("test_data/certs/client.pem"));
    config.0.http_cfg.tls.client_key_cert_path = Some(PathBuf::from("test_data/certs/client.key"));
    config.0.http_cfg.tls.ca_tls_cert_path = Some(PathBuf::from("test_data/certs/ca.pem"));
    config.0.http_cfg.tls.client_ca_tls_cert_path =
        Some(PathBuf::from("test_data/certs/client_ca.pem"));
    config.0.http_cfg.tls.enable = true;
    let mut http_app =
        TestApp::new_with_launching_instance_custom_cfg((config.0, config.1), |_| {})
            .await
            .unwrap();
    let _resp = http_app
        .ourchat_api_get("logo")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    http_app.async_drop().await;
}

#[tokio::test]
async fn test_rate_limit() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.http_cfg.rate_limit.enable = true;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();
    let mut limited = false;
    for _ in 0..100 {
        match app
            .ourchat_api_get("status")
            .await
            .expect("failed")
            .error_for_status()
        {
            Ok(_) => {}
            Err(err) => {
                if err.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                    limited = true;
                }
            }
        }
    }
    assert!(limited);
    app.async_drop().await
}
