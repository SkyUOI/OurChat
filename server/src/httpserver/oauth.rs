use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
};
use base::database::DbPool;
use chrono::Utc;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::helper::{generate_random_string, generate_ocid, USER_ID_GENERATOR};
use snowdon::ClassicLayoutSnowflakeExtension;

#[derive(Debug, Deserialize)]
struct OAuthCallbackParams {
    code: String,
    state: String,
}

#[derive(Debug, Serialize)]
struct GitHubTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

pub struct OAuthConfig {
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
}

pub struct OAuthState {
    pub db_pool: DbPool,
    pub oauth_config: OAuthConfig,
}

async fn github_oauth_start(
    State(state): State<Arc<OAuthState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let state_param = generate_random_string(16);
    
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email&state={}",
        state.oauth_config.github_client_id,
        state.oauth_config.github_redirect_uri,
        state_param
    );
    
    Ok(Redirect::to(&auth_url))
}

async fn github_oauth_callback(
    State(state): State<Arc<OAuthState>>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<impl IntoResponse, StatusCode> {
    // Exchange code for access token
    let token_response = exchange_code_for_token(&state.oauth_config, &params.code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Get user info from GitHub
    let user_info = get_github_user_info(&token_response.access_token).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create or update user in database
    let user = create_or_update_user_from_github(&state.db_pool, user_info).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // TODO: Generate JWT token and return it
    // For now, just return success
    Ok("GitHub OAuth successful. User created/updated.")
}

async fn exchange_code_for_token(
    config: &OAuthConfig,
    code: &str,
) -> Result<GitHubTokenResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let token_request = GitHubTokenRequest {
        client_id: config.github_client_id.clone(),
        client_secret: config.github_client_secret.clone(),
        code: code.to_string(),
        redirect_uri: config.github_redirect_uri.clone(),
    };
    
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&token_request)
        .send()
        .await?;
    
    let token_response: GitHubTokenResponse = response.json().await?;
    Ok(token_response)
}

async fn get_github_user_info(access_token: &str) -> Result<GitHubUserInfo, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("User-Agent", "OurChat")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    let user_info: GitHubUserInfo = response.json().await?;
    Ok(user_info)
}

async fn create_or_update_user_from_github(
    db_pool: &DbPool,
    github_user: GitHubUserInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    use entities::user::Entity as UserEntity;
    use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, ActiveValue::Set};
    
    let github_id = github_user.id.to_string();
    
    // Check if user already exists with this GitHub ID
    let existing_user = UserEntity::find()
        .filter(entities::user::Column::GithubId.eq(&github_id))
        .one(&db_pool.db_pool)
        .await?;
    
    if let Some(user) = existing_user {
        // Update existing user
        let mut user_active: entities::user::ActiveModel = user.into();
        user_active.name = Set(github_user.name.unwrap_or_else(|| github_user.login.clone()));
        user_active.email = Set(github_user.email.unwrap_or_default());
        user_active.avatar = Set(github_user.avatar_url);
        user_active.oauth_provider = Set(Some("github".to_string()));
        user_active.update_time = Set(Utc::now().into());
        
        UserEntity::update(user_active).exec(&db_pool.db_pool).await?;
    } else {
        // Create new user
        let user_id = USER_ID_GENERATOR.generate()?.into_i64();
        let new_user = entities::user::ActiveModel {
            id: Set(user_id),
            ocid: Set(generate_ocid(10)),
            passwd: Set(None), // OAuth users don't have passwords
            name: Set(github_user.name.unwrap_or_else(|| github_user.login.clone())),
            email: Set(github_user.email.unwrap_or_default()),
            time: Set(Utc::now().into()),
            resource_used: Set(0),
            friend_limit: Set(5000),
            friends_num: Set(0),
            avatar: Set(github_user.avatar_url),
            public_update_time: Set(Utc::now().into()),
            update_time: Set(Utc::now().into()),
            account_status: Set(1), // Active
            deleted_at: Set(None),
            public_key: Set(vec![]), // TODO: Generate public key for OAuth users
            github_id: Set(github_id),
            oauth_provider: Set(Some("github".to_string())),
        };
        
        UserEntity::insert(new_user).exec(&db_pool.db_pool).await?;
    }
    
    Ok(())
}

pub fn config() -> axum::Router<Arc<OAuthState>> {
    axum::Router::new()
        .route("/oauth/github", get(github_oauth_start))
        .route("/oauth/github/callback", get(github_oauth_callback))
}
