use crate::db::friend::query_friend;
use crate::process::error_msg::{self, not_found};
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::ID;
use pb::service::ourchat::friends::delete_friend::v1::{DeleteFriendRequest, DeleteFriendResponse};
use tonic::{Request, Response, Status};

pub async fn delete_friend(
    server: &RpcServer,
    id: ID,
    request: Request<DeleteFriendRequest>,
) -> Result<Response<DeleteFriendResponse>, Status> {
    match delete_friend_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            DeleteFriendErr::Db(_) | DeleteFriendErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            DeleteFriendErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum DeleteFriendErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn delete_friend_impl(
    server: &RpcServer,
    id: ID,
    request: Request<DeleteFriendRequest>,
) -> Result<DeleteFriendResponse, DeleteFriendErr> {
    // check friendship exist
    let req = request.into_inner();
    let friend_id: ID = req.friend_id.into();
    let friend_info = query_friend(id, friend_id, &server.db.db_pool).await?;
    if friend_info.is_none() {
        Err(Status::not_found(not_found::FRIEND))?;
    }
    if let Err(e) = db::friend::delete_friend(id, friend_id, &server.db.db_pool).await {
        match e {
            db::friend::DeleteFriendError::FriendShipNotFound => {
                Err(Status::not_found(error_msg::not_found::FRIEND))?
            }
            db::friend::DeleteFriendError::Db(db_err) => Err(db_err)?,
        }
    }
    let ret = DeleteFriendResponse {};
    Ok(ret)
}
