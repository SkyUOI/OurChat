use crate::db::manager;
use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use base::consts::ID;
use pb::service::server_manage::user_manage::v1::{
    AssignServerRoleRequest, AssignServerRoleResponse,
};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum AssignServerRoleError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
}

async fn assign_server_role_impl(
    server: &ServerManageServiceProvider,
    request: Request<AssignServerRoleRequest>,
) -> Result<AssignServerRoleResponse, AssignServerRoleError> {
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
            Err(Status::internal(SERVER_ERROR))
        }
    }
}
