use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use pb::ourchat::session::ban::v1::{
    BanUserRequest, BanUserResponse, UnbanUserRequest, UnbanUserResponse,
};
use tonic::{Request, Response, Status};

pub async fn ban_user(
    server: &RpcServer,
    request: Request<BanUserRequest>,
) -> Result<Response<BanUserResponse>, Status> {
    match ban_user_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            BanUserErr::Db(_) | BanUserErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            BanUserErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum BanUserErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] tonic::Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn ban_user_impl(
    server: &RpcServer,
    request: Request<BanUserRequest>,
) -> Result<BanUserResponse, BanUserErr> {
    todo!()
}

pub async fn unban_user(
    server: &RpcServer,
    request: Request<UnbanUserRequest>,
) -> Result<Response<UnbanUserResponse>, Status> {
    match unban_user_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            BanUserErr::Db(_) | BanUserErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            BanUserErr::Status(status) => Err(status),
        },
    }
}

async fn unban_user_impl(
    server: &RpcServer,
    request: Request<UnbanUserRequest>,
) -> Result<UnbanUserResponse, BanUserErr> {
    todo!()
}
