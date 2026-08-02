use crate::{process, server::ServerManageServiceProvider};
use entities::server_management_role;
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::user_manage::v1::{
    ListServerRolesRequest, ListServerRolesResponse, ServerRoleInfo,
};
use sea_orm::EntityTrait;
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
enum ListServerRolesError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("permission denied")]
    PermissionDenied,
}

async fn list_server_roles_impl(
    server: &ServerManageServiceProvider,
    request: Request<ListServerRolesRequest>,
) -> Result<ListServerRolesResponse, ListServerRolesError> {
    // Authorization: enumerating role definitions is gated by AssignRole.
    process::check_server_manage_permission(
        &request,
        PredefinedServerManagementPermission::AssignRole,
        &server.db,
    )
    .await
    .map_err(|_| ListServerRolesError::PermissionDenied)?;

    let roles = server_management_role::Entity::find()
        .all(&server.db.db_pool)
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
            match e {
                ListServerRolesError::PermissionDenied => Err(Status::permission_denied(
                    crate::process::error_msg::PERMISSION_DENIED,
                )),
                _ => Err(Status::internal(crate::process::error_msg::SERVER_ERROR)),
            }
        }
    }
}
