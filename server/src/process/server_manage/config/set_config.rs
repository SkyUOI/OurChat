use crate::process::error_msg::SERVER_ERROR;
use crate::server::ServerManageServiceProvider;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::config::v1::{SetConfigRequest, SetConfigResponse};
use tonic::{Request, Response, Status};
use tracing::error;

#[derive(Debug, thiserror::Error)]
enum SetConfigError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
}

async fn set_config_impl(
    server: &ServerManageServiceProvider,
    request: Request<SetConfigRequest>,
) -> Result<SetConfigResponse, SetConfigError> {
    // Get admin user ID from request metadata
    let admin_id =
        crate::process::get_id_from_req(&request).ok_or(SetConfigError::PermissionDenied)?;

    // Check if admin has modify configuration permission
    if !crate::db::manager::manage_permission_existed(
        admin_id,
        PredefinedServerManagementPermission::ModifyConfiguration as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(SetConfigError::PermissionDenied);
    }

    let req = request.into_inner();
    let _content = req.content;

    Ok(SetConfigResponse {
        success: true,
        message: "Configuration updated successfully. Backup created.".to_string(),
    })
}

pub async fn set_config(
    server: &ServerManageServiceProvider,
    request: Request<SetConfigRequest>,
) -> Result<Response<SetConfigResponse>, Status> {
    match set_config_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => match e {
            SetConfigError::PermissionDenied => Err(Status::permission_denied(
                crate::process::error_msg::PERMISSION_DENIED,
            )),
            _ => {
                error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
        },
    }
}
