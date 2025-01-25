use crate::db::session::if_permission_exist;
use crate::{
    process::{
        error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found::USER_IN_SESSION},
        get_id_from_req,
    },
    server::RpcServer,
};
use migration::m20241229_022701_add_role_for_session::PreDefinedPermissions;
use pb::service::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
use sea_orm::{ActiveModelTrait, ActiveValue};
use tonic::{Request, Response, Status};

pub async fn set_role(
    server: &RpcServer,
    request: Request<SetRoleRequest>,
) -> Result<Response<SetRoleResponse>, Status> {
    match set_role_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            SetRoleErr::Db(_) | SetRoleErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
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
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn set_role_impl(
    server: &RpcServer,
    request: Request<SetRoleRequest>,
) -> Result<SetRoleResponse, SetRoleErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    // check the privilege
    if !if_permission_exist(
        id,
        req.session_id.into(),
        PreDefinedPermissions::SetRole.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(SetRoleErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    let model = entities::user_role_relation::ActiveModel {
        user_id: ActiveValue::Set(id.into()),
        role_id: ActiveValue::Set(req.role_id as i64),
        session_id: ActiveValue::Set(req.session_id as i64),
    };
    // update
    match model.update(&server.db.db_pool).await {
        Ok(_) => {}
        Err(sea_orm::DbErr::RecordNotUpdated) => {
            // record does not exist, create it
            // There is something wrong,
            // because every member in the session should have a role
            return Err(SetRoleErr::Status(Status::not_found(USER_IN_SESSION)));
        }
        Err(e) => return Err(SetRoleErr::Db(e)),
    }

    let ret = SetRoleResponse {};
    Ok(ret)
}
