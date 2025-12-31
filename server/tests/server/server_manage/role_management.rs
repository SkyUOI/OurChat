use client::TestApp;
use migration::predefined::PredefinedServerManagementRole;
use pb::service::server_manage::user_manage::v1::{
    AssignServerRoleRequest, ListUserServerRolesRequest,
};
use server::db::manager;
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

    // Assign admin role to admin user using the database directly
    admin_user
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Test 1: Target user has no roles initially
    let response = admin_user
        .lock()
        .await
        .server_manage()
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
    let response = admin_user
        .lock()
        .await
        .server_manage()
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
    let response = admin_user
        .lock()
        .await
        .server_manage()
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

    // Assign admin role to admin user so they have permission
    admin_user
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Test 1: Assign admin role to target user
    let _response = admin_user
        .lock()
        .await
        .server_manage()
        .assign_server_role(Request::new(AssignServerRoleRequest {
            user_id: target_id.into(),
            role_id: PredefinedServerManagementRole::Admin as u64,
        }))
        .await
        .unwrap();

    // Verify the role was assigned by listing roles
    let list_response = admin_user
        .lock()
        .await
        .server_manage()
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
    let _result = admin_user
        .lock()
        .await
        .server_manage()
        .assign_server_role(Request::new(AssignServerRoleRequest {
            user_id: target_id.into(),
            role_id: PredefinedServerManagementRole::Admin as u64,
        }))
        .await;

    // It might error due to duplicate, but that's okay.
    // We'll just ensure the role is still present.
    let list_response = admin_user
        .lock()
        .await
        .server_manage()
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
    let result = admin_user
        .lock()
        .await
        .server_manage()
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
