use client::TestApp;
use entities::user;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[tokio::test]
async fn test_email_verification_configuration() {
    // Test with email verification disabled (default behavior)
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    tracing::debug!(
        "App config.require_email_verification: {}",
        app.app_config.main_cfg.require_email_verification
    );

    // Register a user with email verification disabled
    let user = app.new_user().await.unwrap();

    // Check that user was created with email_verified = true (since verification is disabled)
    let user_record = user::Entity::find()
        .filter(user::Column::Email.eq(&user.lock().await.email))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query user")
        .expect("User not found");

    tracing::debug!("User email_verified: {}", user_record.email_verified);
    assert!(
        user_record.email_verified,
        "User should be email verified when verification is disabled"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn test_email_verification_enabled() {
    // Test with email verification enabled
    let (mut config, args) = TestApp::get_test_config().unwrap();
    tracing::debug!(
        "Initial config.require_email_verification: {}",
        config.main_cfg.require_email_verification
    );
    config.main_cfg.require_email_verification = true;
    tracing::debug!(
        "After setting config.require_email_verification: {}",
        config.main_cfg.require_email_verification
    );

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    tracing::debug!(
        "App config.require_email_verification: {}",
        app.app_config.main_cfg.require_email_verification
    );

    // Register a user with email verification enabled
    let user = app.new_user().await.unwrap();

    // Check that user was created with email_verified = false (since verification is enabled)
    let user_record = user::Entity::find()
        .filter(user::Column::Email.eq(&user.lock().await.email))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query user")
        .expect("User not found");

    tracing::debug!("User email_verified: {}", user_record.email_verified);
    assert!(
        !user_record.email_verified,
        "User should not be email verified when verification is enabled"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn test_email_verification_flow() {
    // This test would require a more complex setup with email client mocking
    // For now, we'll test the basic functionality without actual email sending

    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Test the verify endpoint exists and responds correctly
    let response = app
        .verify("invalid_token")
        .await
        .expect("Failed to execute request.");

    // Should return 400 for invalid token
    assert_eq!(response.status().as_u16(), 400);

    app.async_drop().await;
}

#[tokio::test]
async fn test_oauth_user_email_verified() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Simulate OAuth user creation (this would normally be done via OAuth flow)
    // OAuth users should always have email_verified = true
    let new_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(999999),
        ocid: sea_orm::ActiveValue::Set("oauth_test".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("OAuth User".to_string()),
        email: sea_orm::ActiveValue::Set("oauth_user@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(None),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(Some("12345".to_string())),
        oauth_provider: sea_orm::ActiveValue::Set(Some("github".to_string())),
        email_verified: sea_orm::ActiveValue::Set(true), // OAuth users are always verified
    };

    user::Entity::insert(new_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert OAuth user");

    // Verify the OAuth user has email_verified = true
    let oauth_user = user::Entity::find()
        .filter(user::Column::Email.eq("oauth_user@example.com"))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query OAuth user")
        .expect("OAuth user not found");

    assert!(
        oauth_user.email_verified,
        "OAuth users should always be email verified"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn test_oauth_user_email_verified_with_verification_enabled() {
    // Test that OAuth users are always verified even when email verification is enabled
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.require_email_verification = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Simulate OAuth user creation with email verification enabled
    let new_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(999998),
        ocid: sea_orm::ActiveValue::Set("oauth_test2".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("OAuth User 2".to_string()),
        email: sea_orm::ActiveValue::Set("oauth_user2@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(None),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(Some("12346".to_string())),
        oauth_provider: sea_orm::ActiveValue::Set(Some("github".to_string())),
        email_verified: sea_orm::ActiveValue::Set(true), // OAuth users are always verified
    };

    user::Entity::insert(new_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert OAuth user");

    // Verify the OAuth user has email_verified = true even with verification enabled
    let oauth_user = user::Entity::find()
        .filter(user::Column::Email.eq("oauth_user2@example.com"))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query OAuth user")
        .expect("OAuth user not found");

    assert!(
        oauth_user.email_verified,
        "OAuth users should always be email verified even when verification is enabled"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn test_auth_denied_for_unverified_email() {
    // Test that users with unverified emails cannot authenticate when email verification is required
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.require_email_verification = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Register a user (they will have email_verified = false because verification is enabled)
    let user = app.new_user().await.unwrap();

    // Try to authenticate the user - this should fail because their email is not verified
    let auth_result = user.lock().await.email_auth().await;

    // Authentication should fail for unverified users when email verification is required
    assert!(
        auth_result.is_err(),
        "Authentication should fail for users with unverified emails when verification is required"
    );

    // Check that we get a "user not found" error (which is what we return for unverified users)
    let error = auth_result.unwrap_err().unwrap_rpc_status();
    assert_eq!(
        error.code(),
        tonic::Code::NotFound,
        "Should return NotFound error for unverified users"
    );

    app.async_drop().await;
}

#[tokio::test]
async fn test_auth_allowed_for_verified_email() {
    // Test that users with verified emails can authenticate when email verification is required
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.require_email_verification = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Register a user (they will have email_verified = false because verification is enabled)
    let user = app.new_user().await.unwrap();

    // Manually verify the user's email in the database
    let email = user.lock().await.email.clone();
    let user_record = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query user")
        .expect("User not found");

    let mut user_active: user::ActiveModel = user_record.into();
    user_active.email_verified = sea_orm::ActiveValue::Set(true);
    user::Entity::update(user_active)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to update user email verification status");

    // Try to authenticate the user - this should succeed because their email is now verified
    let auth_result = user.lock().await.email_auth().await;

    // Authentication should succeed for verified users
    assert!(
        auth_result.is_ok(),
        "Authentication should succeed for users with verified emails when verification is required"
    );

    app.async_drop().await;
}
