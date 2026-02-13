use crate::db::session::in_session;
use crate::process::error_msg::{NOT_IN_SESSION, SERVER_ERROR, not_found};
use crate::server::RpcServer;
use base::constants::{ID, SessionID};
use base::types::RoleId;
use pb::service::ourchat::session::get_role::v1::{GetRoleRequest, GetRoleResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tonic::{Request, Response, Status};

pub async fn get_role(
    server: &RpcServer,
    id: ID,
    request: Request<GetRoleRequest>,
) -> Result<Response<GetRoleResponse>, Status> {
    match get_role_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            GetRoleErr::Db(_) | GetRoleErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            GetRoleErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum GetRoleErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn get_role_impl(
    server: &RpcServer,
    id: ID,
    request: Request<GetRoleRequest>,
) -> Result<GetRoleResponse, GetRoleErr> {
    let req = request.into_inner();
    let role_id = RoleId(req.role_id);
    let model = match entities::role::Entity::find_by_id(role_id.0 as i64)
        .one(&server.db.db_pool)
        .await?
    {
        Some(data) => data,
        None => Err(Status::not_found(not_found::ROLE))?,
    };
    let session_id: Option<SessionID> = model.session_id.map(|x| x.into());
    if let Some(session_id) = session_id
        && !in_session(id, session_id, &server.db.db_pool).await?
    {
        Err(Status::permission_denied(NOT_IN_SESSION))?;
    }
    let permissions = entities::role_permissions::Entity::find()
        .filter(entities::role_permissions::Column::RoleId.eq(req.role_id))
        .all(&server.db.db_pool)
        .await?;
    let ret = GetRoleResponse {
        name: model.name,
        description: model.description,
        permissions: permissions.iter().map(|x| x.permission_id as u64).collect(),
        session_id: model.session_id.map(|x| x as u64),
    };
    Ok(ret)
}
