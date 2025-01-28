use crate::process::{error_msg::SERVER_ERROR, get_id_from_req};
use crate::server::RpcServer;
use pb::ourchat::session::add_role::v1::{AddRoleRequest, AddRoleResponse};
use sea_orm::{ActiveModelTrait, ActiveValue};
use tonic::{Request, Response, Status};

pub async fn add_role(
    server: &RpcServer,
    request: Request<AddRoleRequest>,
) -> Result<Response<AddRoleResponse>, Status> {
    match add_role_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AddRoleErr::Db(_) | AddRoleErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AddRoleErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AddRoleErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn add_role_impl(
    server: &RpcServer,
    request: Request<AddRoleRequest>,
) -> Result<AddRoleResponse, AddRoleErr> {
    let id = get_id_from_req(&request).unwrap();
    let model = entities::role::ActiveModel {
        creator_id: ActiveValue::Set(Some(id.into())),
        description: ActiveValue::Set(request.into_inner().description),
        ..Default::default()
    };
    let model = model.insert(&server.db.db_pool).await?;
    let res = AddRoleResponse {
        role_id: model.id as u64,
    };
    Ok(res)
}
