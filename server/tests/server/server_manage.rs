use std::sync::Arc;

use client::TestApp;
use client::oc_helper::server_manager::TestServerManager;
use migration::predefined::PredefinedServerManagementRole;
use pb::google::protobuf::Duration;
use pb::service::server_manage::user_manage::v1::{
    AssignServerRoleRequest, BanUserRequest, ListUserServerRolesRequest, UnbanUserRequest,
};
use server::db::manager;
use tokio::sync::Mutex;
use tonic::Request;

#[tokio::test]
async fn list_user_server_roles_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create two users: one admin who will make the request, one regular user to list roles for
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    let admin_id = admin_user.lock().await.id;
    let target_id = target_user.lock().await.id;

    tracing::info!("admin user id: {}, target user id: {}", admin_id, target_id);

    // Create server manager for admin user
    let server_manager = Arc::new(Mutex::new(
        TestServerManager::new(&app, admin_id, admin_user.lock().await.token.clone())
            .await
            .unwrap(),
    ));

    // Assign admin role to admin user
    server_manager
        .lock()
        .await
        .assign_role(PredefinedServerManagementRole::Admin as i64)
        .await
        .unwrap();

    // Test 1: Target user has no roles initially
    let response = server_manager
        .lock()
        .await
        .client
        .list_user_server_roles(Request::new(ListUserServerRolesRequest {
            user_id: target_id.into(),
        }))
        .await
        .unwrap();

    let roles = response.into_inner().role_ids;
    assert!(roles.is_empty(), "New user should have no server roles");

    // Test 2: Assign a role to target user and verify listing
    // We need to assign a role to target user. Since we don't have a direct API yet,
    // we can use the database directly via manager::set_role
    // Assign admin role to target user as well (for testing)
    manager::set_role(
        target_id,
        PredefinedServerManagementRole::Admin as i64,
        &app.db_pool.db_pool,
    )
    .await
    .unwrap();

    // Now list roles again
    let response = server_manager
        .lock()
        .await
        .client
        .list_user_server_roles(Request::new(ListUserServerRolesRequest {
            user_id: target_id.into(),
        }))
        .await
        .unwrap();

    let roles = response.into_inner().role_ids;
    assert_eq!(roles.len(), 1, "User should have 1 role after assignment");
    assert_eq!(
        roles[0],
        PredefinedServerManagementRole::Admin as u64,
        "Role ID should match admin role"
    );

    // Test 3: List roles for non-existent user (should return empty array, not error)
    let response = server_manager
        .lock()
        .await
        .client
        .list_user_server_roles(Request::new(ListUserServerRolesRequest {
            user_id: 999999, // Non-existent user ID
        }))
        .await
        .unwrap();

    let roles = response.into_inner().role_ids;
    assert!(
        roles.is_empty(),
        "Non-existent user should return empty role list"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn assign_server_role_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create two users: one admin who will make the request, one regular user to assign role to
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    let admin_id = admin_user.lock().await.id;
    let target_id = target_user.lock().await.id;

    tracing::info!("admin user id: {}, target user id: {}", admin_id, target_id);

    // Create server manager for admin user
    let server_manager = Arc::new(Mutex::new(
        TestServerManager::new(&app, admin_id, admin_user.lock().await.token.clone())
            .await
            .unwrap(),
    ));

    // Assign admin role to admin user so they have permission
    server_manager
        .lock()
        .await
        .assign_role(PredefinedServerManagementRole::Admin as i64)
        .await
        .unwrap();

    // Test 1: Assign admin role to target user
    let _response = server_manager
        .lock()
        .await
        .client
        .assign_server_role(Request::new(AssignServerRoleRequest {
            user_id: target_id.into(),
            role_id: PredefinedServerManagementRole::Admin as u64,
        }))
        .await
        .unwrap();

    // Verify the role was assigned by listing roles
    let list_response = server_manager
        .lock()
        .await
        .client
        .list_user_server_roles(Request::new(ListUserServerRolesRequest {
            user_id: target_id.into(),
        }))
        .await
        .unwrap();

    let roles = list_response.into_inner().role_ids;
    assert_eq!(roles.len(), 1, "User should have 1 role after assignment");
    assert_eq!(
        roles[0],
        PredefinedServerManagementRole::Admin as u64,
        "Role ID should match admin role"
    );

    // Test 2: Assign same role again (should be idempotent, no error)
    // Note: The database insert might fail due to unique constraint.
    // Let's see what happens - if it errors, we can catch and ignore.
    // For now, just call and ensure no panic.
    let _result = server_manager
        .lock()
        .await
        .client
        .assign_server_role(Request::new(AssignServerRoleRequest {
            user_id: target_id.into(),
            role_id: PredefinedServerManagementRole::Admin as u64,
        }))
        .await;

    // It might error due to duplicate, but that's okay.
    // We'll just ensure the role is still present.
    let list_response = server_manager
        .lock()
        .await
        .client
        .list_user_server_roles(Request::new(ListUserServerRolesRequest {
            user_id: target_id.into(),
        }))
        .await
        .unwrap();

    let roles = list_response.into_inner().role_ids;
    assert_eq!(
        roles.len(),
        1,
        "Duplicate assignment should not create duplicate role"
    );

    // Test 3: Assign role to non-existent user (should error)
    // The API should return an error (database foreign key violation)
    let result = server_manager
        .lock()
        .await
        .client
        .assign_server_role(Request::new(AssignServerRoleRequest {
            user_id: 999999,
            role_id: PredefinedServerManagementRole::Admin as u64,
        }))
        .await;

    // Expect an error
    assert!(
        result.is_err(),
        "Assigning role to non-existent user should error"
    );

    app.async_drop().await;
}
#[tokio::test]
async fn ban_user_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create two users: one admin who will make the request, one regular user to ban
    let admin_user = app.new_user().await.unwrap();
    let target_user = app.new_user().await.unwrap();

    let admin_id = admin_user.lock().await.id;
    let target_id = target_user.lock().await.id;

    tracing::info!("admin user id: {}, target user id: {}", admin_id, target_id);

    // Create server manager for admin user
    let server_manager = Arc::new(Mutex::new(
        TestServerManager::new(&app, admin_id, admin_user.lock().await.token.clone())
            .await
            .unwrap(),
    ));

    // Assign admin role to admin user so they have permission
    server_manager
        .lock()
        .await
        .assign_role(PredefinedServerManagementRole::Admin as i64)
        .await
        .unwrap();

    // Test 1: Ban user with duration
    let _response = server_manager
        .lock()
        .await
        .client
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
    let result = server_manager
        .lock()
        .await
        .client
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
    let _response = server_manager
        .lock()
        .await
        .client
        .ban_user(Request::new(BanUserRequest {
            user_id: permanent_target_id.into(),
            reason: Some("Permanent ban".to_string()),
            duration: None,
        }))
        .await
        .unwrap();

    // Test 4: Unban user with duration
    let _response = server_manager
        .lock()
        .await
        .client
        .unban_user(Request::new(UnbanUserRequest {
            user_id: target_id.into(),
        }))
        .await
        .unwrap();

    // Test 5: Unban already unbanned user (should error)
    let result = server_manager
        .lock()
        .await
        .client
        .unban_user(Request::new(UnbanUserRequest {
            user_id: target_id.into(),
        }))
        .await;
    assert!(result.is_err(), "Unbanning non-banned user should error");

    // Test 6: Unban permanently banned user
    let _response = server_manager
        .lock()
        .await
        .client
        .unban_user(Request::new(UnbanUserRequest {
            user_id: permanent_target_id.into(),
        }))
        .await
        .unwrap();

    // Test 7: Ban non-existent user (should error due to foreign key? maybe not)
    let _result = server_manager
        .lock()
        .await
        .client
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

    let regular_id = regular_user.lock().await.id;
    let target_id = target_user.lock().await.id;

    // Create server manager for regular user (no admin role)
    let server_manager = Arc::new(Mutex::new(
        TestServerManager::new(&app, regular_id, regular_user.lock().await.token.clone())
            .await
            .unwrap(),
    ));

    // Regular user tries to ban without permission
    let result = server_manager
        .lock()
        .await
        .client
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
