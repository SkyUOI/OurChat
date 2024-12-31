use crate::{component::EmailSender, process::get_id_from_req, server::RpcServer};
use pb::ourchat::session::set_session_info::v1::{SetSessionInfoRequest, SetSessionInfoResponse};
use tonic::{Request, Response, Status};

pub async fn set_session_info(
    server: &RpcServer<impl EmailSender>,
    request: Request<SetSessionInfoRequest>,
) -> Result<Response<SetSessionInfoResponse>, Status> {
    match set_session_info_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            let status = match e {
                SetSessionErr::Db(e) => {
                    tracing::error!("Database error: {}", e);
                    Status::internal(e.to_string())
                }
                SetSessionErr::Status(s) => s,
                SetSessionErr::Internal(e) => {
                    tracing::error!("Internal error: {}", e);
                    Status::internal(e.to_string())
                }
            };
            Err(status)
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum SetSessionErr {
    #[error("database error:{0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0}")]
    Status(#[from] tonic::Status),
    #[error("internal error:{0}")]
    Internal(#[from] anyhow::Error),
}

async fn set_session_info_impl(
    server: &RpcServer<impl EmailSender>,
    request: Request<SetSessionInfoRequest>,
) -> Result<SetSessionInfoResponse, SetSessionErr> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    let res = SetSessionInfoResponse {};
    Ok(res)
}
