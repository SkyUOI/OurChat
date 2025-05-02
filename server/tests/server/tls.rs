use std::path::PathBuf;

use client::TestApp;

#[tokio::test]
async fn test_tls() {
    test_tls_on(false).await;
    test_tls_on(true).await;
}

async fn test_tls_on(client_cert_required: bool) {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    config.main_cfg.tls.server_tls_cert_path = Some(PathBuf::from("test_data/certs/server.pem"));
    config.main_cfg.tls.server_key_cert_path = Some(PathBuf::from("test_data/certs/server.key"));
    config.main_cfg.tls.client_tls_cert_path = Some(PathBuf::from("test_data/certs/client.pem"));
    config.main_cfg.tls.client_key_cert_path = Some(PathBuf::from("test_data/certs/client.key"));
    config.main_cfg.tls.ca_tls_cert_path = Some(PathBuf::from("test_data/certs/ca.pem"));
    config.main_cfg.tls.client_ca_tls_cert_path =
        Some(PathBuf::from("test_data/certs/client_ca.pem"));
    config.main_cfg.tls.enable = true;
    config.main_cfg.tls.client_certificate_required = client_cert_required;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args))
        .await
        .unwrap();
    let _user = app.new_user().await.unwrap();
    app.async_drop().await;
}
