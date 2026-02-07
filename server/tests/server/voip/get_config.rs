use client::TestApp;
use pb::service::basic::voip::v1::GetVoipConfigRequest;

/// Tests getting VoIP configuration with default values.
///
/// Verifies:
/// - STUN servers are returned (including Google STUN)
/// - TURN is disabled with empty credentials
/// - TURN TTL is set to 24 hours
/// - Multiple calls return consistent results
#[tokio::test]
async fn get_voip_config() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // First call to get config
    let response1 = user
        .lock()
        .await
        .basic()
        .get_voip_config(GetVoipConfigRequest {})
        .await
        .unwrap()
        .into_inner();

    // Verify STUN servers are returned
    assert!(
        !response1.stun_servers.is_empty(),
        "Should have STUN servers"
    );

    // Check for Google STUN servers (default config)
    let has_google_stun = response1
        .stun_servers
        .iter()
        .any(|s| s.contains("stun.l.google.com"));
    assert!(has_google_stun, "Should include Google STUN servers");

    // Verify TURN is disabled
    assert!(
        !response1.turn_enabled,
        "TURN should be disabled by default"
    );
    assert!(
        response1.turn_server_url.is_empty(),
        "TURN URL should be empty when disabled"
    );
    assert!(
        response1.turn_username.is_empty(),
        "TURN username should be empty when disabled"
    );
    assert!(
        response1.turn_password.is_empty(),
        "TURN password should be empty when disabled"
    );

    // Verify TURN TTL is set to 24 hours
    assert_eq!(response1.turn_ttl, 86400, "TURN TTL should be 24 hours");

    // Second call to verify consistency
    let response2 = user
        .lock()
        .await
        .basic()
        .get_voip_config(GetVoipConfigRequest {})
        .await
        .unwrap()
        .into_inner();

    // Both responses should have the same configuration
    assert_eq!(
        response1.stun_servers, response2.stun_servers,
        "STUN servers should be consistent"
    );
    assert_eq!(
        response1.turn_enabled, response2.turn_enabled,
        "TURN enabled should be consistent"
    );
    assert_eq!(
        response1.turn_ttl, response2.turn_ttl,
        "TURN TTL should be consistent"
    );

    app.async_drop().await;
}
