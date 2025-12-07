use crate::helper::{USER_ID_GENERATOR, generate_ocid, generate_random_string};
use crate::process::generate_access_token;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
};
use base::database::DbPool;
use chrono::Utc;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::sync::Arc;

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
    pub oauth_states: dashmap::DashMap<String, chrono::DateTime<Utc>>,
}

async fn github_oauth_start(
    State(state): State<Arc<OAuthState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let state_param = generate_random_string(16);

    // Store the state with timestamp for CSRF protection
    state.oauth_states.insert(state_param.clone(), Utc::now());

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email&state={}",
        state.oauth_config.github_client_id, state.oauth_config.github_redirect_uri, state_param
    );

    Ok(Redirect::to(&auth_url))
}

async fn github_oauth_callback(
    State(state): State<Arc<OAuthState>>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate state parameter for CSRF protection
    if let Some((_, timestamp)) = state.oauth_states.remove(&params.state) {
        // Check if state is not too old (e.g., 10 minutes)
        if Utc::now() - timestamp > chrono::Duration::minutes(10) {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Exchange code for access token
    let token_response = exchange_code_for_token(&state.oauth_config, &params.code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Log token type and scope for debugging
    tracing::debug!(
        "GitHub OAuth token received - type: {}, scope: {}",
        token_response.token_type,
        token_response.scope
    );

    // Get user info from GitHub
    let user_info = get_github_user_info(&token_response.access_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create or update user in database
    let user_id = create_or_update_user_from_github(&state.db_pool, user_info)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT token
    let token = generate_access_token(user_id.into());

    // Return success with JWT token
    Ok(format!("GitHub OAuth successful. Token: Bearer {}", token))
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

async fn get_github_user_info(
    access_token: &str,
) -> Result<GitHubUserInfo, Box<dyn std::error::Error>> {
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
) -> Result<i64, Box<dyn std::error::Error>> {
    use entities::user::Entity as UserEntity;
    use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

    let github_id = github_user.id.to_string();

    // Check if user already exists with this GitHub ID
    let existing_user = UserEntity::find()
        .filter(entities::user::Column::GithubId.eq(&github_id))
        .one(&db_pool.db_pool)
        .await?;

    let user_id = if let Some(user) = existing_user {
        // Update existing user
        let user_id = user.id;
        let mut user_active: entities::user::ActiveModel = user.into();
        user_active.name = Set(github_user
            .name
            .unwrap_or_else(|| github_user.login.clone()));
        user_active.email = Set(github_user.email.unwrap_or_default());
        user_active.avatar = Set(github_user.avatar_url);
        user_active.oauth_provider = Set(Some("github".to_string()));
        user_active.update_time = Set(Utc::now().into());

        UserEntity::update(user_active)
            .exec(&db_pool.db_pool)
            .await?;
        user_id
    } else {
        // Create new user
        let user_id = USER_ID_GENERATOR.generate()?.into_i64();
        let new_user = entities::user::ActiveModel {
            id: Set(user_id),
            ocid: Set(generate_ocid(10)),
            passwd: Set(None), // OAuth users don't have passwords
            name: Set(github_user
                .name
                .unwrap_or_else(|| github_user.login.clone())),
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
            github_id: Set(Some(github_id)),
            oauth_provider: Set(Some("github".to_string())),
            email_verified: Set(true), // OAuth users from trusted providers are automatically verified
        };

        UserEntity::insert(new_user).exec(&db_pool.db_pool).await?;
        user_id
    };

    Ok(user_id)
}

pub fn config() -> axum::Router<Arc<OAuthState>> {
    axum::Router::new()
        .route("/oauth/github", get(github_oauth_start))
        .route("/oauth/github/callback", get(github_oauth_callback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use http::StatusCode;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_github_oauth_start() {
        let state = Arc::new(OAuthState {
            db_pool: create_mock_db_pool(),
            oauth_config: OAuthConfig {
                github_client_id: "test_client_id".to_string(),
                github_client_secret: "test_client_secret".to_string(),
                github_redirect_uri: "http://localhost:7777/oauth/github/callback".to_string(),
            },
            oauth_states: dashmap::DashMap::new(),
        });

        let response = github_oauth_start(State(state.clone())).await.unwrap();
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("https://github.com/login/oauth/authorize"));
        assert!(location.contains("client_id=test_client_id"));
        assert!(location.contains("redirect_uri=http://localhost:7777/oauth/github/callback"));

        // Check that state was stored
        let state_param = extract_state_from_url(location);
        assert!(state.oauth_states.contains_key(&state_param));
    }

    #[tokio::test]
    async fn test_github_oauth_callback_invalid_state() {
        let state = Arc::new(OAuthState {
            db_pool: create_mock_db_pool(),
            oauth_config: OAuthConfig {
                github_client_id: "test_client_id".to_string(),
                github_client_secret: "test_client_secret".to_string(),
                github_redirect_uri: "http://localhost:7777/oauth/github/callback".to_string(),
            },
            oauth_states: dashmap::DashMap::new(),
        });

        let params = OAuthCallbackParams {
            code: "test_code".to_string(),
            state: "invalid_state".to_string(),
        };

        let result = github_oauth_callback(State(state), axum::extract::Query(params)).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_github_oauth_callback_expired_state() {
        let state = Arc::new(OAuthState {
            db_pool: create_mock_db_pool(),
            oauth_config: OAuthConfig {
                github_client_id: "test_client_id".to_string(),
                github_client_secret: "test_client_secret".to_string(),
                github_redirect_uri: "http://localhost:7777/oauth/github/callback".to_string(),
            },
            oauth_states: dashmap::DashMap::new(),
        });

        // Add an expired state (11 minutes old)
        let expired_time = Utc::now() - chrono::Duration::minutes(11);
        state
            .oauth_states
            .insert("expired_state".to_string(), expired_time);

        let params = OAuthCallbackParams {
            code: "test_code".to_string(),
            state: "expired_state".to_string(),
        };

        let result = github_oauth_callback(State(state), axum::extract::Query(params)).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_state_parameter_generation() {
        let state1 = generate_random_string(16);
        let state2 = generate_random_string(16);

        assert_eq!(state1.len(), 16);
        assert_eq!(state2.len(), 16);
        assert_ne!(state1, state2); // Should be different each time
    }

    // Helper function to extract state parameter from GitHub OAuth URL
    fn extract_state_from_url(url: &str) -> String {
        let url_parts: Vec<&str> = url.split('&').collect();
        for part in url_parts {
            if let Some(stripped) = part.strip_prefix("state=") {
                return stripped.to_string();
            }
        }
        panic!("State parameter not found in URL: {}", url);
    }

    // Create a mock database pool for testing
    fn create_mock_db_pool() -> DbPool {
        // For unit tests, we don't need actual database connections
        // since we're testing the OAuth logic, not database operations
        let redis_config = deadpool_redis::Config::from_url("redis://localhost:6379");
        DbPool {
            db_pool: sea_orm::DatabaseConnection::Disconnected,
            redis_pool: redis_config
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .unwrap(),
        }
    }
}
