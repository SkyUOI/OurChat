use crate::process::error_msg::SERVER_ERROR;
use crate::server::ServerManageServiceProvider;
use anyhow::Context;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::config::v1::{GetConfigRequest, GetConfigResponse};
use tonic::{Request, Response, Status};
use tracing::error;

#[derive(Debug, thiserror::Error)]
enum GetConfigError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
}

async fn get_config_impl(
    server: &ServerManageServiceProvider,
    request: Request<GetConfigRequest>,
) -> Result<GetConfigResponse, GetConfigError> {
    // Get admin user ID from request metadata
    let admin_id =
        crate::process::get_id_from_req(&request).ok_or(GetConfigError::PermissionDenied)?;

    // Check if admin has view configuration permission
    if !crate::db::manager::manage_permission_existed(
        admin_id,
        PredefinedServerManagementPermission::ViewConfiguration as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(GetConfigError::PermissionDenied);
    }
    let content = serde_json::to_string(&*server.shared_data.cfg())
        .context("Fatal: cannot serialize config to json")?;
    Ok(GetConfigResponse { content })
}

pub async fn get_config(
    server: &ServerManageServiceProvider,
    request: Request<GetConfigRequest>,
) -> Result<Response<GetConfigResponse>, Status> {
    match get_config_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => match e {
            GetConfigError::PermissionDenied => Err(Status::permission_denied(
                crate::process::error_msg::PERMISSION_DENIED,
            )),
            _ => {
                error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}
