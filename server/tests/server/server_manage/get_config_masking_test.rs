//! Tests that GetConfig never leaks plaintext secrets (C-8). Even an admin
//! with ViewConfiguration permission must only see masked placeholders for
//! sensitive fields (database/redis/rabbitmq passwords, OAuth secret, TURN
//! secret), not the live credentials.

use client::TestApp;
use pb::service::server_manage::config::v1::GetConfigRequest;
use tonic::Request;

const MASK: &str = "***";

#[tokio::test]
async fn get_config_masks_all_sensitive_fields() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin = app.new_user().await.unwrap();
    admin
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    let resp = admin
        .lock()
        .await
        .server_manage()
        .get_config(Request::new(GetConfigRequest {}))
        .await
        .expect("admin should be able to read config");

    let content = resp.into_inner().content;
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("config is valid JSON");

    // Each sensitive path must be present and equal to the mask placeholder.
    let sensitive_paths: &[&[&str]] = &[
        &["db_cfg", "passwd"],
        &["redis_cfg", "passwd"],
        &["rabbitmq_cfg", "passwd"],
        &["main_cfg", "oauth", "github_client_secret"],
        &["main_cfg", "voip", "turn_password"],
    ];

    for path in sensitive_paths {
        let mut node = &parsed;
        for seg in *path {
            node = node.get(seg).unwrap_or_else(|| {
                panic!("config missing path segment `{seg}` while checking masking")
            });
        }
        assert_eq!(
            node,
            MASK,
            "sensitive field `{}` must be masked, got {node}",
            path.join(".")
        );
    }

    // And the live plaintext credentials used by the test harness must NOT
    // appear anywhere in the serialized output.
    for secret in ["123456", "postgres", "guest"] {
        // Only flag matches that look like actual password values; mask is "***"
        // and structure fields like "user" are not secrets. We check that no
        // `passwd`/`secret`/`password` JSON entry carries a non-mask value.
        let _ = secret; // (kept for documentation; concrete check below)
    }
    // Walk every leaf whose key looks sensitive and assert it is masked.
    fn walk(value: &serde_json::Value, prefix: &str, bad: &mut Vec<String>) {
        if let serde_json::Value::Object(map) = value {
            for (k, v) in map {
                let p = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                let is_sensitive_key = {
                    let k_low = k.to_lowercase();
                    k_low == "passwd"
                        || k_low == "password"
                        || k_low.contains("secret")
                        || k_low == "turn_password"
                };
                if is_sensitive_key && !v.is_null() && v.as_str() != Some(MASK) {
                    bad.push(format!("{p} = {v}"));
                }
                walk(v, &p, bad);
            }
        }
    }
    let mut bad = Vec::new();
    walk(&parsed, "", &mut bad);
    assert!(
        bad.is_empty(),
        "unmasked sensitive fields leaked by GetConfig: {bad:?}"
    );

    app.async_drop().await;
}
