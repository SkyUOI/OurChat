use crate::{process::error_msg::SERVER_ERROR, server::ServerManageServiceProvider};
use base::constants::ID;
use entities::manager_role_relation;
use pb::service::server_manage::user_manage::v1::{
    ListUserServerRolesRequest, ListUserServerRolesResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum ListUserServerRolesError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
}

async fn list_user_server_roles_impl(
    server: &ServerManageServiceProvider,
    request: Request<ListUserServerRolesRequest>,
) -> Result<ListUserServerRolesResponse, ListUserServerRolesError> {
    let req = request.into_inner();
    let user_id: ID = req.user_id.into();

    // Query all role IDs for the given user
    let roles = manager_role_relation::Entity::find()
        .filter(manager_role_relation::Column::UserId.eq(user_id))
        .all(&server.db.db_pool)
        .await?;

    // Extract role IDs from the results
    let role_ids: Vec<u64> = roles
        .into_iter()
        .map(|model| model.role_id as u64)
        .collect();

    info!(
        "listed {} server roles for user {}",
        role_ids.len(),
        user_id
    );

    Ok(ListUserServerRolesResponse { role_ids })
}

pub async fn list_user_server_roles(
    server: &ServerManageServiceProvider,
    request: Request<ListUserServerRolesRequest>,
) -> Result<Response<ListUserServerRolesResponse>, Status> {
    match list_user_server_roles_impl(server, request).await {
        Ok(response) => Ok(Response::new(response)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal(SERVER_ERROR))
        }
    }
}
