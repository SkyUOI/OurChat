use crate::{component::EmailSender, server::RpcServer};
use pb::ourchat::session::accept_session::v1::{AcceptSessionRequest, AcceptSessionResponse};
use tonic::Response;

#[derive(Debug, thiserror::Error)]
enum AcceptSessionError {
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    UnknownError(#[from] anyhow::Error),
}

async fn accept_impl(
    _server: &RpcServer<impl EmailSender>,
    _request: tonic::Request<AcceptSessionRequest>,
) -> Result<AcceptSessionResponse, AcceptSessionError> {
    // TODO:check if the time is expired
    Ok(AcceptSessionResponse {})
}

pub async fn accept_session(
    server: &RpcServer<impl EmailSender>,
    request: tonic::Request<AcceptSessionRequest>,
) -> Result<Response<AcceptSessionResponse>, tonic::Status> {
    match accept_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            match e {
                AcceptSessionError::DbError(db_err) => {
                    tracing::error!("{}", db_err);
                }
                AcceptSessionError::UnknownError(error) => {
                    tracing::error!("{}", error);
                }
            }
            Err(tonic::Status::internal("Server Error"))
        }
    }
}
