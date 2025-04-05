use crate::process::error_msg::{ROLE_NAME_EMPTY, SERVER_ERROR};
use crate::server::RpcServer;
use base::consts::ID;
use pb::service::ourchat::session::add_role::v1::{AddRoleRequest, AddRoleResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, TransactionTrait};
use tonic::{Request, Response, Status};

pub async fn add_role(
    server: &RpcServer,
    id: ID,
    request: Request<AddRoleRequest>,
) -> Result<Response<AddRoleResponse>, Status> {
    match add_role_impl(server, id, request).await {
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
    id: ID,
    request: Request<AddRoleRequest>,
) -> Result<AddRoleResponse, AddRoleErr> {
    let req = request.into_inner();

    // validate name
    if req.name.trim().is_empty() {
        return Err(AddRoleErr::Status(Status::invalid_argument(
            ROLE_NAME_EMPTY,
        )));
    }

    // begin transaction
    let txn = server.db.db_pool.begin().await?;

    // create role
    let model = entities::role::ActiveModel {
        creator_id: ActiveValue::Set(Some(id.into())),
        description: ActiveValue::Set(req.description),
        name: ActiveValue::Set(req.name),
        ..Default::default()
    };

    let role = model.insert(&txn).await?;

    // add permissions
    entities::role_permissions::Entity::insert_many(req.permissions.into_iter().map(|x| {
        entities::role_permissions::ActiveModel {
            role_id: ActiveValue::Set(role.id),
            permission_id: ActiveValue::Set(x),
        }
    }))
    .exec(&txn)
    .await?;

    // commit transaction
    txn.commit().await?;

    let res = AddRoleResponse { role_id: role.id };
    Ok(res)
}
