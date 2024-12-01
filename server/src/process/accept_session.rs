use crate::{
    component::EmailSender,
    pb::ourchat::session::accept_session::v1::{AcceptSessionRequest, AcceptSessionResponse},
    server::RpcServer,
};
use tonic::Response;

#[derive(Debug, thiserror::Error)]
enum AcceptSessionError {
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    UnknownError(#[from] anyhow::Error),
}

async fn accept_impl(
    server: &RpcServer<impl EmailSender>,
) -> Result<AcceptSessionResponse, AcceptSessionError> {
    Ok(AcceptSessionResponse {})
}

pub async fn accept_session(
    server: &RpcServer<impl EmailSender>,
    request: tonic::Request<AcceptSessionRequest>,
) -> Result<tonic::Response<AcceptSessionResponse>, tonic::Status> {
    // check if the time is expired
    Ok(Response::new(AcceptSessionResponse {}))
}
