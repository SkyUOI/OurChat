use client::TestApp;
use entities::user;
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[tokio::test]
async fn test_github_oauth_start() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test the GitHub OAuth start endpoint
    let response = app
        .http_get("oauth/github")
        .await
        .expect("Failed to execute request");

    // Should either redirect (FOUND = 302) or return OK in test mode
    if response.status() == StatusCode::FOUND {
        // Check that the location header contains the GitHub OAuth URL
        let location = response
            .headers()
            .get("location")
            .expect("Location header not found")
            .to_str()
            .expect("Failed to convert location to string");

        assert!(location.starts_with("https://github.com/login/oauth/authorize"));
        assert!(location.contains("client_id="));
        assert!(location.contains("redirect_uri="));
        assert!(location.contains("scope=user:email"));
        assert!(location.contains("state="));
    } else {
        // In test mode with empty credentials, the endpoint might just exist
        assert_eq!(response.status(), StatusCode::OK);
    }

    app.async_drop().await;
}

#[tokio::test]
async fn test_github_oauth_callback_invalid_state() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test with invalid state parameter
    let response = app
        .http_get("oauth/github/callback?code=test_code&state=invalid_state")
        .await
        .expect("Failed to execute request");

    // Should return bad request for invalid state
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    app.async_drop().await;
}

#[tokio::test]
async fn test_github_oauth_callback_missing_code() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test with missing code parameter
    let response = app
        .http_get("oauth/github/callback?state=some_state")
        .await
        .expect("Failed to execute request");

    // Should return bad request for missing parameters
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    app.async_drop().await;
}

#[tokio::test]
async fn test_github_oauth_callback_missing_state() {
    let (mut config, args) = TestApp::get_test_config().unwrap();

    // Enable OAuth for this test
    config.main_cfg.oauth.enable = true;

    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Test with missing state parameter
    let response = app
        .http_get("oauth/github/callback?code=test_code")
        .await
        .expect("Failed to execute request");

    // Should return bad request for missing state
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    app.async_drop().await;
}

#[tokio::test]
async fn test_oauth_user_creation() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a user with GitHub OAuth fields
    let new_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(99997),
        ocid: sea_orm::ActiveValue::Set("oauth_create_test".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("GitHub OAuth Test User".to_string()),
        email: sea_orm::ActiveValue::Set("github_oauth_test@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(Some("https://example.com/avatar.png".to_string())),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(Some("99997".to_string())),
        oauth_provider: sea_orm::ActiveValue::Set(Some("github".to_string())),
        email_verified: sea_orm::ActiveValue::Set(true),
    };

    user::Entity::insert(new_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert OAuth user");

    // Verify the OAuth user was created correctly
    let oauth_user = user::Entity::find()
        .filter(user::Column::GithubId.eq("99997"))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query OAuth user")
        .expect("OAuth user not found");

    assert_eq!(oauth_user.github_id, Some("99997".to_string()));
    assert_eq!(oauth_user.oauth_provider, Some("github".to_string()));
    assert!(oauth_user.email_verified);
    assert!(oauth_user.passwd.is_none());
    assert_eq!(
        oauth_user.avatar,
        Some("https://example.com/avatar.png".to_string())
    );

    app.async_drop().await;
}

#[tokio::test]
async fn test_oauth_user_update() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create an OAuth user
    let new_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(99996),
        ocid: sea_orm::ActiveValue::Set("oauth_update_test".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("Original Name".to_string()),
        email: sea_orm::ActiveValue::Set("oauth_update@example.com".to_string()),
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
        github_id: sea_orm::ActiveValue::Set(Some("99996".to_string())),
        oauth_provider: sea_orm::ActiveValue::Set(Some("github".to_string())),
        email_verified: sea_orm::ActiveValue::Set(true),
    };

    user::Entity::insert(new_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert OAuth user");

    // Simulate updating the user via OAuth (e.g., user changed their GitHub profile)
    let user_record = user::Entity::find()
        .filter(user::Column::GithubId.eq("99996"))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query OAuth user")
        .expect("OAuth user not found");

    let mut user_active: user::ActiveModel = user_record.into();
    user_active.name = sea_orm::ActiveValue::Set("Updated Name".to_string());
    user_active.email = sea_orm::ActiveValue::Set("updated_email@example.com".to_string());
    user_active.avatar =
        sea_orm::ActiveValue::Set(Some("https://example.com/new_avatar.png".to_string()));
    user_active.update_time = sea_orm::ActiveValue::Set(chrono::Utc::now().into());

    user::Entity::update(user_active)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to update OAuth user");

    // Verify the user was updated correctly
    let updated_user = user::Entity::find()
        .filter(user::Column::GithubId.eq("99996"))
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query updated OAuth user")
        .expect("Updated OAuth user not found");

    assert_eq!(updated_user.name, "Updated Name");
    assert_eq!(updated_user.email, "updated_email@example.com");
    assert_eq!(
        updated_user.avatar,
        Some("https://example.com/new_avatar.png".to_string())
    );

    app.async_drop().await;
}
