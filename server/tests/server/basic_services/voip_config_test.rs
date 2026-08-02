//! Tests that GetVoipConfig requires authentication and, when the optional
//! `turn_static_auth_secret` is configured, returns per-user time-limited
//! credentials instead of the static shared password (C-5).

use base::constants::JWT_HEADER;
use client::TestApp;
use pb::service::basic::voip::v1::GetVoipConfigRequest;
use tonic::{Code, Request};

#[tokio::test]
async fn voip_config_requires_authentication() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Use a registered user's basic client but deliberately send the request
    // WITHOUT attaching the JWT — anonymous access must be rejected.
    let anonymous = app.new_user().await.unwrap();
    let err = anonymous
        .lock()
        .await
        .basic()
        .get_voip_config(Request::new(GetVoipConfigRequest {}))
        .await
        .expect_err("anonymous GetVoipConfig must be rejected");
    assert!(
        matches!(err.code(), Code::Unauthenticated | Code::PermissionDenied),
        "expected Unauthenticated/PermissionDenied, got {:?}",
        err
    );

    app.async_drop().await;
}

#[tokio::test]
async fn voip_config_succeeds_with_token() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Attach the JWT to the request metadata manually (BasicService has no
    // auth interceptor on the test client).
    let token = user.lock().await.token.clone();
    let mut req = Request::new(GetVoipConfigRequest {});
    req.metadata_mut().insert(
        JWT_HEADER,
        format!("Bearer {token}")
            .parse()
            .expect("valid header value"),
    );

    let resp = user
        .lock()
        .await
        .basic()
        .get_voip_config(req)
        .await
        .expect("authenticated GetVoipConfig must succeed");
    // Response is well-formed (STUN list present, even if empty).
    let _ = resp.into_inner();

    app.async_drop().await;
}

#[tokio::test]
async fn voip_config_returns_ephemeral_credentials_when_secret_set() {
    // Start the server with a TURN static-auth-secret so credentials are
    // generated per-user instead of echoing the static password.
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.voip.turn_enabled = true;
    config.main_cfg.voip.turn_server_url = "turn:example.com:3478".to_string();
    config.main_cfg.voip.turn_username = "static-user".to_string();
    config.main_cfg.voip.turn_password = "static-shared-secret".to_string();
    config.main_cfg.voip.turn_static_auth_secret = Some("topsecret".to_string());
    config.main_cfg.voip.turn_ttl = 3600;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();

    let token = user.lock().await.token.clone();
    let mut req = Request::new(GetVoipConfigRequest {});
    req.metadata_mut()
        .insert(JWT_HEADER, format!("Bearer {token}").parse().unwrap());
    let resp = user
        .lock()
        .await
        .basic()
        .get_voip_config(req)
        .await
        .expect("GetVoipConfig must succeed");
    let inner = resp.into_inner();

    // The returned username must NOT be the static one; it encodes
    // "<expiry_unix>:<user_id>" per the coturn REST API.
    assert!(inner.turn_enabled);
    assert_ne!(
        inner.turn_username, "static-user",
        "ephemeral credentials must replace the static username"
    );
    assert_ne!(
        inner.turn_password, "static-shared-secret",
        "the static shared secret must never be returned"
    );
    let (expiry_str, uid_str) = inner
        .turn_username
        .split_once(':')
        .expect("ephemeral username must be `<expiry>:<userid>`");
    let expiry: i64 = expiry_str.parse().expect("expiry is an integer");
    let uid: u64 = uid_str.parse().expect("user id is an integer");
    let now = chrono::Utc::now().timestamp();
    assert!(expiry > now, "credential must be valid in the future");
    assert!(
        expiry <= now + 3600 + 5,
        "credential ttl must respect turn_ttl"
    );
    assert_eq!(uid, *user.lock().await.id);

    app.async_drop().await;
}
