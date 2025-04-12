use std::path::PathBuf;

use client::TestApp;

#[tokio::test]
async fn test_tls() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    config.main_cfg.tls_cert_path = Some(PathBuf::from("tests/test_data/ca.pem"));
    config.main_cfg.key_cert_path = Some(PathBuf::from("tests/test_data/ca.key"));
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args))
        .await
        .unwrap();
    app.async_drop().await;
}
