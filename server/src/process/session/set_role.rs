use crate::db::session::if_permission_exist;
use crate::{
    process::error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found::USER_IN_SESSION},
    server::RpcServer,
};
use base::constants::ID;
use base::types::RoleId;
use migration::predefined::PredefinedPermissions;
use pb::service::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
use sea_orm::{ActiveModelTrait, ActiveValue};
use tonic::{Request, Response, Status};

pub async fn set_role(
    server: &RpcServer,
    id: ID,
    request: Request<SetRoleRequest>,
) -> Result<Response<SetRoleResponse>, Status> {
    match set_role_impl(server, id, request).await {
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
    id: ID,
    request: Request<SetRoleRequest>,
) -> Result<SetRoleResponse, SetRoleErr> {
    let req = request.into_inner();
    let member_id: ID = req.member_id.into();
    // check the privilege
    if !if_permission_exist(
        id,
        req.session_id.into(),
        PredefinedPermissions::SetRole.into(),
        &server.db.db_pool,
    )
    .await?
    {
        Err(Status::permission_denied(PERMISSION_DENIED))?;
    }
    let role_id = RoleId(req.role_id);
    let model = entities::user_role_relation::ActiveModel {
        user_id: ActiveValue::Set(member_id.into()),
        role_id: ActiveValue::Set(role_id.0 as i64),
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
