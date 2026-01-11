use super::{
    Files,
    error_msg::{PERMISSION_DENIED, SERVER_ERROR},
};
use crate::server::RpcServer;
use base::consts::ID;
use base::database::DbPool;
use entities::user;
use pb::service::ourchat::delete::v1::{DeleteFileRequest, DeleteFileResponse};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use size::Size;
use std::path::PathBuf;
use tokio::fs;
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
pub enum DeleteFileError {
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("permission denied")]
    PermissionDenied,
    #[error("Internal IO error:{0:?}")]
    InternalIOError(#[from] std::io::Error),
}

async fn delete_file_impl(
    id: ID,
    req: DeleteFileRequest,
    db_conn: &DbPool,
) -> Result<DeleteFileResponse, DeleteFileError> {
    // First check if the file exists and get file info
    let file_info = match Files::find_by_id(&req.key).one(&db_conn.db_pool).await? {
        Some(f) => f,
        None => return Err(DeleteFileError::PermissionDenied),
    };

    // Check if user has permission:
    // User owns the file
    let has_permission = if file_info.user_id == i64::from(id) {
        true // File owner
    } else {
        false // Neither owner nor in a session
    };

    if !has_permission {
        return Err(DeleteFileError::PermissionDenied);
    }

    // Get file size before deleting
    let path = PathBuf::from(&file_info.path);
    let file_size = match fs::metadata(&path).await {
        Ok(metadata) => Size::from_bytes(metadata.len()),
        Err(_) => Size::from_bytes(0),
    };

    // Delete the file from filesystem
    if path.exists() {
        fs::remove_file(&path).await?;
    }

    // Update user's resource usage
    if let Some(user) = user::Entity::find_by_id(id).one(&db_conn.db_pool).await? {
        let current_usage = user.resource_used as u64;
        let file_size_bytes = file_size.bytes() as u64;
        let new_usage = current_usage.saturating_sub(file_size_bytes);
        let mut user_active: user::ActiveModel = user.into();
        user_active.resource_used = Set(new_usage as i64);
        user_active.update(&db_conn.db_pool).await?;
    }

    // Delete the file record from database
    Files::delete_by_id(&req.key).exec(&db_conn.db_pool).await?;

    Ok(DeleteFileResponse {})
}

pub async fn delete_file(
    server: &RpcServer,
    id: ID,
    request: Request<DeleteFileRequest>,
) -> Result<Response<DeleteFileResponse>, Status> {
    let req = request.into_inner();
    match delete_file_impl(id, req, &server.db).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => match e {
            DeleteFileError::PermissionDenied => Err(Status::permission_denied(PERMISSION_DENIED)),
            _ => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}
