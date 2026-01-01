use crate::db::redis;
use crate::process::error_msg::not_found::NOT_BE_BANNED;
use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use anyhow::Context;
use base::consts::ID;
use deadpool_redis::redis::AsyncCommands;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::user_manage::v1::{UnbanUserRequest, UnbanUserResponse};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum UnbanUserError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("redis error:{0:?}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
    #[error("user not banned")]
    NotBanned,
}

async fn unban_user_impl(
    server: &ServerManageServiceProvider,
    request: Request<UnbanUserRequest>,
) -> Result<UnbanUserResponse, UnbanUserError> {
    // Get admin user ID from request metadata
    let admin_id =
        crate::process::get_id_from_req(&request).ok_or(UnbanUserError::PermissionDenied)?;

    // Check if admin has ban permission (unban uses same permission as ban)
    if !crate::db::manager::manage_permission_existed(
        admin_id,
        PredefinedServerManagementPermission::BanUser as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(UnbanUserError::PermissionDenied);
    }

    let req = request.into_inner();
    let user_id: ID = req.user_id.into();

    // Get Redis connection
    let mut conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;

    let key = redis::map_server_ban_to_redis(user_id);

    // Check if user is actually banned
    let exists: bool = conn.exists(&key).await?;
    if !exists {
        return Err(UnbanUserError::NotBanned);
    }

    // Remove ban key
    let _: () = conn.del(&key).await?;

    // Also remove reason key if it exists
    let reason_key = format!("{}:reason", key);
    let _: () = conn.del(&reason_key).await?;

    info!("unbanned user {}", user_id);

    Ok(UnbanUserResponse {})
}

pub async fn server_unban_user(
    server: &ServerManageServiceProvider,
    request: Request<UnbanUserRequest>,
) -> Result<Response<UnbanUserResponse>, Status> {
    match unban_user_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            match e {
                UnbanUserError::PermissionDenied => Err(Status::permission_denied(
                    crate::process::error_msg::PERMISSION_DENIED,
                )),
                UnbanUserError::NotBanned => Err(Status::not_found(NOT_BE_BANNED)),
                _ => Err(Status::internal(SERVER_ERROR)),
            }
        }
    }
}
