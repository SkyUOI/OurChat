use crate::{process, server::ServerManageServiceProvider};
use entities::{server_management_permission, server_management_role_permissions};
use migration::predefined::PredefinedServerManagementPermission;
use pb::service::server_manage::user_manage::v1::{
    ListServerRolePermissionsRequest, ListServerRolePermissionsResponse, PermissionInfo,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

#[derive(Debug, thiserror::Error)]
enum ListServerRolePermissionsError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("permission denied")]
    PermissionDenied,
}

async fn list_server_role_permissions_impl(
    server: &ServerManageServiceProvider,
    request: Request<ListServerRolePermissionsRequest>,
) -> Result<ListServerRolePermissionsResponse, ListServerRolePermissionsError> {
    // Authorization: enumerating the permissions attached to a role is gated by
    // AssignRole.
    process::check_server_manage_permission(
        &request,
        PredefinedServerManagementPermission::AssignRole,
        &server.db,
    )
    .await
    .map_err(|_| ListServerRolePermissionsError::PermissionDenied)?;

    let req = request.into_inner();
    let role_id = req.role_id as i64;

    let role_permissions = server_management_role_permissions::Entity::find()
        .filter(server_management_role_permissions::Column::RoleId.eq(role_id))
        .all(&server.db.db_pool)
        .await?;

    let permission_ids: Vec<i64> = role_permissions
        .into_iter()
        .map(|model| model.permission_id)
        .collect();

    let permissions = server_management_permission::Entity::find()
        .filter(server_management_permission::Column::Id.is_in(permission_ids))
        .all(&server.db.db_pool)
        .await?;

    let permission_infos: Vec<PermissionInfo> = permissions
        .into_iter()
        .map(|model| PermissionInfo {
            id: model.id as u64,
            name: model.name,
            description: model.description.unwrap_or_default(),
        })
        .collect();

    Ok(ListServerRolePermissionsResponse {
        permissions: permission_infos,
    })
}

pub async fn list_server_role_permissions(
    server: &ServerManageServiceProvider,
    request: Request<ListServerRolePermissionsRequest>,
) -> Result<Response<ListServerRolePermissionsResponse>, Status> {
    match list_server_role_permissions_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            match e {
                ListServerRolePermissionsError::PermissionDenied => Err(Status::permission_denied(
                    crate::process::error_msg::PERMISSION_DENIED,
                )),
                _ => Err(Status::internal(crate::process::error_msg::SERVER_ERROR)),
            }
        }
    }
}
