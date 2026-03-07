use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use entities::server_management_role;
use pb::service::server_manage::user_manage::v1::{
    ListServerRolesRequest, ListServerRolesResponse, ServerRoleInfo,
};
use sea_orm::EntityTrait;
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
enum ListServerRolesError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
}

async fn list_server_roles_impl(
    _server: &ServerManageServiceProvider,
    _request: Request<ListServerRolesRequest>,
) -> Result<ListServerRolesResponse, ListServerRolesError> {
    let roles = server_management_role::Entity::find()
        .all(&_server.db.db_pool)
        .await?;

    let role_infos: Vec<ServerRoleInfo> = roles
        .into_iter()
        .map(|model| ServerRoleInfo {
            id: model.id as u64,
            name: model.name,
            description: model.description.unwrap_or_default(),
        })
        .collect();

    Ok(ListServerRolesResponse { roles: role_infos })
}

pub async fn list_server_roles(
    server: &ServerManageServiceProvider,
    request: Request<ListServerRolesRequest>,
) -> Result<Response<ListServerRolesResponse>, Status> {
    match list_server_roles_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal(SERVER_ERROR))
        }
    }
}
