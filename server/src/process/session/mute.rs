use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use pb::ourchat::session::mute::v1::{
    MuteUserRequest, MuteUserResponse, UnmuteUserRequest, UnmuteUserResponse,
};
use tonic::{Request, Response, Status};

pub async fn mute_user(
    server: &RpcServer,
    request: Request<MuteUserRequest>,
) -> Result<Response<MuteUserResponse>, Status> {
    match mute_user_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            MuteUserErr::Db(_) | MuteUserErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            MuteUserErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum MuteUserErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] tonic::Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn mute_user_impl(
    server: &RpcServer,
    request: Request<MuteUserRequest>,
) -> Result<MuteUserResponse, MuteUserErr> {
    todo!()
}

pub async fn unmute_user(
    server: &RpcServer,
    request: Request<UnmuteUserRequest>,
) -> Result<Response<UnmuteUserResponse>, Status> {
    match unmute_user_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            MuteUserErr::Db(_) | MuteUserErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            MuteUserErr::Status(status) => Err(status),
        },
    }
}

async fn unmute_user_impl(
    server: &RpcServer,
    request: Request<UnmuteUserRequest>,
) -> Result<UnmuteUserResponse, MuteUserErr> {
    todo!()
}
