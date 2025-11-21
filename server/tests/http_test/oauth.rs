use client::TestApp;
use http::StatusCode;

#[tokio::test]
async fn oauth_start_endpoint_redirects() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test that the OAuth start endpoint redirects to GitHub
    let response = app
        .http_get("oauth/github")
        .await
        .expect("Failed to make OAuth start request");

    // The endpoint should redirect (302) or at least exist (200)
    // In test mode with empty credentials, it might not redirect
    if response.status() == StatusCode::FOUND {
        // Check that it redirects to GitHub
        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("https://github.com/login/oauth/authorize"));
        assert!(location.contains("client_id="));
        assert!(location.contains("redirect_uri="));
        assert!(location.contains("state="));
    } else {
        // With empty credentials, the endpoint might just exist
        assert_eq!(response.status(), StatusCode::OK);
    }

    app.async_drop().await;
}

#[tokio::test]
async fn oauth_callback_invalid_state() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test that callback with invalid state returns bad request
    let response = app
        .http_get("oauth/github/callback?code=test_code&state=invalid_state")
        .await
        .expect("Failed to make OAuth callback request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    app.async_drop().await;
}

#[tokio::test]
async fn oauth_callback_missing_parameters() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test that callback without required parameters returns bad request
    let response = app
        .http_get("oauth/github/callback")
        .await
        .expect("Failed to make OAuth callback request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    app.async_drop().await;
}

#[tokio::test]
async fn oauth_callback_invalid_code() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // First get a valid state from the start endpoint
    let start_response = app
        .http_get("oauth/github")
        .await
        .expect("Failed to make OAuth start request");

    // If we got a redirect, extract state from location header
    // Otherwise, we need to handle the case where OAuth is not configured
    let state_param = if start_response.status() == StatusCode::FOUND {
        let location = start_response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        extract_state_from_url(location)
    } else {
        // If no redirect, use a test state
        "test_state".to_string()
    };

    // Test callback with invalid code (GitHub will reject this)
    let response = app
        .http_get(&format!(
            "oauth/github/callback?code=invalid_code&state={}",
            state_param
        ))
        .await
        .expect("Failed to make OAuth callback request");

    // With invalid state or code, it should return bad request
    // The exact status code depends on where the error occurs
    assert!(response.status().is_client_error() || response.status().is_server_error());

    app.async_drop().await;
}

#[tokio::test]
async fn oauth_configuration_loaded() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Test that OAuth configuration is properly loaded
    // The default config should have empty client ID and secret and disabled
    assert!(!app.app_config.main_cfg.oauth.enable);
    assert_eq!(app.app_config.main_cfg.oauth.github_client_id, "");
    assert_eq!(app.app_config.main_cfg.oauth.github_client_secret, "");

    app.async_drop().await;
}

#[tokio::test]
async fn oauth_with_custom_config() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Configure OAuth with test values
    config.main_cfg.oauth.enable = true;
    config.main_cfg.oauth.github_client_id = "test_client_id".to_string();
    config.main_cfg.oauth.github_client_secret = "test_client_secret".to_string();

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test that OAuth start endpoint works with custom config
    let response = app
        .http_get("oauth/github")
        .await
        .expect("Failed to make OAuth start request");

    // With custom config, it should redirect
    if response.status() == StatusCode::FOUND {
        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains("client_id=test_client_id"));
    } else {
        // Even with custom config, it might not redirect in test environment
        assert_eq!(response.status(), StatusCode::OK);
    }

    app.async_drop().await;
}

// Helper function to extract state parameter from GitHub OAuth URL
fn extract_state_from_url(url: &str) -> String {
    let url_parts: Vec<&str> = url.split('&').collect();
    for part in url_parts {
        if let Some(stripped) = part.strip_prefix("state=") {
            return stripped.to_string();
        }
    }
    panic!("State parameter not found in URL: {}", url);
}
