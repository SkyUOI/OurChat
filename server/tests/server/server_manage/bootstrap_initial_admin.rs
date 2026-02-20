use base::constants::OCID;
use client::TestApp;
use migration::predefined::PredefinedServerManagementPermission;
use server::db::manager;

#[tokio::test]
async fn bootstrap_initial_admin_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a user first - this user will become admin
    let target_user = app.new_user().await.unwrap();
    let target_ocid_string = target_user.lock().await.ocid.clone();

    tracing::info!("Target user OCID: {}", target_ocid_string);

    // Verify the user does NOT have admin role initially
    let has_admin_before = manager::manage_permission_existed(
        target_user.lock().await.id,
        PredefinedServerManagementPermission::BanUser as i64,
        app.get_db_connection(),
    )
    .await
    .unwrap();
    assert!(
        !has_admin_before,
        "User should not be admin before bootstrap"
    );

    // Now call bootstrap_initial_admin directly to assign admin role
    let result = manager::bootstrap_initial_admin(
        &Some(target_ocid_string.clone()),
        app.get_db_connection(),
    )
    .await
    .unwrap();

    assert!(
        result.is_some(),
        "Should return Some(ocid) when user is found"
    );
    assert_eq!(
        result.unwrap(),
        target_ocid_string,
        "Should return the same OCID"
    );

    // Verify the user now HAS admin role
    let has_admin_after = manager::manage_permission_existed(
        target_user.lock().await.id,
        PredefinedServerManagementPermission::BanUser as i64,
        app.get_db_connection(),
    )
    .await
    .unwrap();
    assert!(has_admin_after, "User should be admin after bootstrap");

    // Test: calling again should be idempotent (user already admin)
    let result2 = manager::bootstrap_initial_admin(
        &Some(target_ocid_string.clone()),
        app.get_db_connection(),
    )
    .await
    .unwrap();

    assert!(result2.is_some(), "Should still return Some(ocid)");
    assert_eq!(
        result2.unwrap(),
        target_ocid_string,
        "Should return the same OCID"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn bootstrap_initial_admin_none_config_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Test with None config - should return None without error
    let result = manager::bootstrap_initial_admin(&None, app.get_db_connection())
        .await
        .unwrap();

    assert!(result.is_none(), "Should return None when config is None");

    app.async_drop().await;
}

#[tokio::test]
async fn bootstrap_initial_admin_user_not_found_test() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Test with non-existent OCID - should warn but not error
    let result = manager::bootstrap_initial_admin(
        &Some(OCID("non_existent_ocid_12345".to_string())),
        app.get_db_connection(),
    )
    .await
    .unwrap();

    assert!(result.is_none(), "Should return None when user not found");

    app.async_drop().await;
}
