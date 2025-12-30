use crate::db::redis;
use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use anyhow::Context;
use base::consts::ID;
use deadpool_redis::redis::AsyncCommands;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::user_manage::v1::{BanUserRequest, BanUserResponse};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum BanUserError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("redis error:{0:?}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
}

async fn ban_user_impl(
    server: &ServerManageServiceProvider,
    request: Request<BanUserRequest>,
) -> Result<BanUserResponse, BanUserError> {
    // Get admin user ID from request metadata
    let admin_id =
        crate::process::get_id_from_req(&request).ok_or(BanUserError::PermissionDenied)?;

    // Check if admin has ban permission
    if !crate::db::manager::manage_permission_existed(
        admin_id,
        PredefinedServerManagementPermission::BanUser as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(BanUserError::PermissionDenied);
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

    // Store ban with optional TTL
    match req.duration {
        Some(duration) => {
            let _: () = conn.set_ex(&key, "1", duration.seconds as u64).await?;
            info!(
                "banned user {} for {} seconds, reason: {:?}",
                user_id, duration.seconds, req.reason
            );
        }
        None => {
            let _: () = conn.set(&key, "1").await?;
            info!(
                "permanently banned user {}, reason: {:?}",
                user_id, req.reason
            );
        }
    }

    // Optionally store reason in separate key if provided
    if let Some(reason) = req.reason {
        let reason_key = format!("{}:reason", key);
        let _: () = conn.set(&reason_key, reason).await?;
        // If there's a duration, set TTL on reason key as well
        if let Some(duration) = req.duration {
            let _: () = conn.expire(&reason_key, duration.seconds).await?;
        }
    }

    Ok(BanUserResponse {})
}

pub async fn server_ban_user(
    server: &ServerManageServiceProvider,
    request: Request<BanUserRequest>,
) -> Result<Response<BanUserResponse>, Status> {
    match ban_user_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            match e {
                BanUserError::PermissionDenied => Err(Status::permission_denied(
                    crate::process::error_msg::PERMISSION_DENIED,
                )),
                _ => Err(Status::internal(SERVER_ERROR)),
            }
        }
    }
}
