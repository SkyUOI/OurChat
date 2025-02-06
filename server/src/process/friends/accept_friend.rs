use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use pb::service::ourchat::friends::accept_friend::v1::{AcceptFriendRequest, AcceptFriendResponse};
use tonic::{Request, Response, Status};

pub async fn accept_friend(
    server: &RpcServer,
    request: Request<AcceptFriendRequest>,
) -> Result<Response<AcceptFriendResponse>, Status> {
    match accept_friend_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AcceptFriendErr::Db(_) | AcceptFriendErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptFriendErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AcceptFriendErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn accept_friend_impl(
    server: &RpcServer,
    request: Request<AcceptFriendRequest>,
) -> Result<AcceptFriendResponse, AcceptFriendErr> {
    let ret = AcceptFriendResponse {};
    Ok(ret)
}
