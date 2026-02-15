use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use base::{constants::ID, database::DbPool};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{SharedData, db::user::get_account_info_db};

#[derive(Debug, Serialize, Deserialize)]
pub struct AvatarParams {
    user_id: ID,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("user ID not found")]
    UserIdNotFound,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::Internal(e) => {
                tracing::error!("Internal server error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Something went wrong: {}", e),
                )
                    .into_response()
            }
            AppError::UserIdNotFound => {
                (StatusCode::NOT_FOUND, "User ID not found").into_response()
            }
        }
    }
}

#[axum::debug_handler]
pub async fn avatar(
    State((pool, shared_data)): State<(DbPool, Arc<SharedData>)>,
    Query(params): Query<AvatarParams>,
) -> Result<impl IntoResponse, AppError> {
    let user = match get_account_info_db(params.user_id, &pool.db_pool)
        .await
        .context("db error")?
    {
        Some(u) => u,
        None => return Err(AppError::UserIdNotFound),
    };
    match user.avatar {
        Some(avatar_key) => {
            // Use hierarchical storage path for better filesystem performance
            let base_path = shared_data.cfg().main_cfg.files_storage_path.clone();
            let path = crate::db::file_storage::generate_hierarchical_path(
                &base_path,
                params.user_id,
                &avatar_key,
            );
            let bytes = tokio::fs::read(&path)
                .await
                .with_context(|| format!("read avatar file failed: {}", path.display()))?;
            Ok(bytes)
        }
        None => {
            // use default avatar
            let path = shared_data.cfg().http_cfg.default_avatar_path.clone();
            let bytes = tokio::fs::read(path)
                .await
                .context("read default avatar failed")?;
            Ok(bytes)
        }
    }
}
