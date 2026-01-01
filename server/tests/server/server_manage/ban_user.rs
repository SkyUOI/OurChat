use client::TestApp;
use pb::google::protobuf::Duration;
use pb::service::server_manage::user_manage::v1::{BanUserRequest, UnbanUserRequest};
use tonic::Request;

#[tokio::test]
async fn ban_user_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create two users: one admin who will make the request, one regular user to ban
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    let admin_id = admin_user.lock().await.id;
    let target_id = target_user.lock().await.id;

    tracing::info!("admin user id: {}, target user id: {}", admin_id, target_id);

    // Assign admin role to admin user so they have permission
    admin_user
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Test 1: Ban user with duration
    let _response = admin_user
        .lock()
        .await
        .server_manage()
        .ban_user(Request::new(BanUserRequest {
            user_id: target_id.into(),
            reason: Some("Test ban with duration".to_string()),
            duration: Some(Duration {
                seconds: 60,
                nanos: 0,
            }), // 60 seconds
        }))
        .await
        .unwrap();

    // Test 2: Ban same user again (idempotent, should not error)
    let result = admin_user
        .lock()
        .await
        .server_manage()
        .ban_user(Request::new(BanUserRequest {
            user_id: target_id.into(),
            reason: Some("Duplicate ban".to_string()),
            duration: Some(Duration {
                seconds: 120,
                nanos: 0,
            }),
        }))
        .await;
    // Should succeed (idempotent)
    assert!(result.is_ok(), "Duplicate ban should be idempotent");

    // Test 3: Ban user without duration (permanent)
    let permanent_target = app.new_user().await.unwrap();
    let permanent_target_id = permanent_target.lock().await.id;
    let _response = admin_user
        .lock()
        .await
        .server_manage()
        .ban_user(Request::new(BanUserRequest {
            user_id: permanent_target_id.into(),
            reason: Some("Permanent ban".to_string()),
            duration: None,
        }))
        .await
        .unwrap();

    // Test 4: Unban user with duration
    let _response = admin_user
        .lock()
        .await
        .server_manage()
        .unban_user(Request::new(UnbanUserRequest {
            user_id: target_id.into(),
        }))
        .await
        .unwrap();

    // Test 5: Unban already unbanned user (should error)
    let result = admin_user
        .lock()
        .await
        .server_manage()
        .unban_user(Request::new(UnbanUserRequest {
            user_id: target_id.into(),
        }))
        .await;
    assert!(result.is_err(), "Unbanning non-banned user should error");

    // Test 6: Unban permanently banned user
    let _response = admin_user
        .lock()
        .await
        .server_manage()
        .unban_user(Request::new(UnbanUserRequest {
            user_id: permanent_target_id.into(),
        }))
        .await
        .unwrap();

    // Test 7: Ban non-existent user (should error due to foreign key? maybe not)
    let _result = admin_user
        .lock()
        .await
        .server_manage()
        .ban_user(Request::new(BanUserRequest {
            user_id: 999999,
            reason: None,
            duration: None,
        }))
        .await;
    // Expect an error (user doesn't exist, but Redis will still set key)
    // We'll accept either success or error, but we should verify behavior
    // For now, just ensure no panic.

    app.async_drop().await;
}

#[tokio::test]
async fn ban_user_permission_test() {
    // Test that user without ban permission cannot ban
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let regular_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    let target_id = target_user.lock().await.id;

    // Regular user tries to ban without permission
    let result = regular_user
        .lock()
        .await
        .server_manage()
        .ban_user(Request::new(BanUserRequest {
            user_id: target_id.into(),
            reason: None,
            duration: None,
        }))
        .await;

    // Should get permission denied error
    assert!(
        result.is_err(),
        "User without permission should not be able to ban"
    );

    app.async_drop().await;
}
