use crate::{component::EmailSender, server::RpcServer};
use pb::ourchat::session::get_session_info::v1::{GetSessionInfoRequest, GetSessionInfoResponse};
use tonic::{Request, Response, Status};

pub async fn get_session_info(
    server: &RpcServer<impl EmailSender>,
    request: Request<GetSessionInfoRequest>,
) -> Result<Response<GetSessionInfoResponse>, Status> {
    // process::get_session_info(self, request).await
    todo!()
}

#[derive(Debug, thiserror::Error)]
enum GetSessionErr {
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("status error:{0}")]
    StatusError(#[from] tonic::Status),
    #[error("internal error:{0}")]
    InternalError(#[from] anyhow::Error),
}

async fn get_session_info_impl(
    server: RpcServer<impl EmailSender>,
    request: Request<GetSessionInfoRequest>,
) -> Result<GetSessionInfoResponse, GetSessionErr> {
    let res = GetSessionInfoResponse::default();
    let req_inner = request.into_inner();

    Ok(res)
}
