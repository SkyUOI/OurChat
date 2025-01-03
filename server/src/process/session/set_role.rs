use crate::{component::EmailSender, process::get_id_from_req, server::RpcServer};
use migration::m20241229_022701_add_role_for_session::PreDefinedPermissions;
use pb::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
use tonic::{Request, Response, Status};

use super::check_if_permission_exist;

pub async fn set_role(
    server: &RpcServer<impl EmailSender>,
    request: Request<SetRoleRequest>,
) -> Result<Response<SetRoleResponse>, Status> {
    match set_role_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            SetRoleErr::Db(_) | SetRoleErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal("Server Error"))
            }
            SetRoleErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum SetRoleErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] tonic::Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn set_role_impl(
    server: &RpcServer<impl EmailSender>,
    request: Request<SetRoleRequest>,
) -> Result<SetRoleResponse, SetRoleErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    // check the privilege
    if !check_if_permission_exist(
        id,
        PreDefinedPermissions::SetRole.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(SetRoleErr::Status(Status::permission_denied(
            "Can't set role",
        )));
    }
    let ret = SetRoleResponse {};
    Ok(ret)
}
