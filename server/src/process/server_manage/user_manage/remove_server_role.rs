use crate::db::manager;
use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use base::constants::ID;
use pb::service::server_manage::user_manage::v1::{
    RemoveServerRoleRequest, RemoveServerRoleResponse,
};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum RemoveServerRoleError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
}

async fn remove_server_role_impl(
    server: &ServerManageServiceProvider,
    request: Request<RemoveServerRoleRequest>,
) -> Result<RemoveServerRoleResponse, RemoveServerRoleError> {
    let req = request.into_inner();
    let user_id: ID = req.user_id.into();
    let role_id: i64 = req.role_id as i64;

    // Remove the role from the user
    manager::remove_role(user_id, role_id, &server.db.db_pool).await?;

    info!("removed server role {} from user {}", role_id, user_id);

    Ok(RemoveServerRoleResponse {})
}

pub async fn remove_server_role(
    server: &ServerManageServiceProvider,
    request: Request<RemoveServerRoleRequest>,
) -> Result<Response<RemoveServerRoleResponse>, Status> {
    match remove_server_role_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal(SERVER_ERROR))
        }
    }
}
