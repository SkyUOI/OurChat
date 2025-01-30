use crate::process::error_msg::exist::FRIEND;
use crate::process::get_id_from_req;
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::ID;
use entities::prelude::Friend;
use pb::service::ourchat::friends::add_friend::v1::{AddFriendRequest, AddFriendResponse};
use sea_orm::EntityTrait;
use tonic::{Request, Response, Status};

pub async fn add_friend(
    server: &RpcServer,
    request: Request<AddFriendRequest>,
) -> Result<Response<AddFriendResponse>, Status> {
    match add_friend_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AddFriendErr::Db(_) | AddFriendErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AddFriendErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AddFriendErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn add_friend_impl(
    server: &RpcServer,
    request: Request<AddFriendRequest>,
) -> Result<AddFriendResponse, AddFriendErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let friend_id: ID = req.friend_id.into();
    let exist = Friend::find_by_id((id.into(), friend_id.into()))
        .one(&server.db.db_pool)
        .await?
        .is_some();
    if exist {
        Err(Status::already_exists(FRIEND))?;
    }
    let ret = AddFriendResponse {};
    Ok(ret)
}
