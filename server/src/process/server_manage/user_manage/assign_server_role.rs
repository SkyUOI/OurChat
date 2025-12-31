use crate::db::manager;
use crate::{
    process::error_msg::{PERMISSION_DENIED, SERVER_ERROR},
    server::ServerManageServiceProvider,
};
use base::consts::ID;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::user_manage::v1::{
    AssignServerRoleRequest, AssignServerRoleResponse,
};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum AssignServerRoleError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("permission denied")]
    PermissionDenied,
}

async fn assign_server_role_impl(
    server: &ServerManageServiceProvider,
    request: Request<AssignServerRoleRequest>,
) -> Result<AssignServerRoleResponse, AssignServerRoleError> {
    // Get requester's user ID from request metadata
    let requester_id =
        crate::process::get_id_from_req(&request).ok_or(AssignServerRoleError::PermissionDenied)?;

    // Check if requester has assign_role permission
    if !crate::db::manager::manage_permission_existed(
        requester_id,
        PredefinedServerManagementPermission::AssignRole as i64,
        &server.db.db_pool,
    )
    .await?
    {
        return Err(AssignServerRoleError::PermissionDenied);
    }

    let req = request.into_inner();
    let user_id: ID = req.user_id.into();
    let role_id: i64 = req.role_id as i64;

    // Assign the role to the user
    manager::set_role(user_id, role_id, &server.db.db_pool).await?;

    info!("assigned server role {} to user {}", role_id, user_id);

    Ok(AssignServerRoleResponse {})
}

pub async fn assign_server_role(
    server: &ServerManageServiceProvider,
    request: Request<AssignServerRoleRequest>,
) -> Result<Response<AssignServerRoleResponse>, Status> {
    match assign_server_role_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            match e {
                AssignServerRoleError::PermissionDenied => {
                    Err(Status::permission_denied(PERMISSION_DENIED))
                }
                _ => Err(Status::internal(SERVER_ERROR)),
            }
        }
    }
}
