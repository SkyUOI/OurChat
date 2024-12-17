use crate::{
    component::EmailSender,
    process::{get_id_from_req, message::del_msg},
    server::RpcServer,
};
use pb::ourchat::msg_delivery::recall::v1::{RecallMsgRequest, RecallMsgResponse};
use tonic::{Request, Response, Status};

use super::DelMsgErr;

pub async fn recall_msg(
    server: &RpcServer<impl EmailSender>,
    request: Request<RecallMsgRequest>,
) -> Result<Response<RecallMsgResponse>, Status> {
    match recall_msg_internal(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(match e {
            RecallErr::Db(db_err) => {
                tracing::error!("{:?}", db_err);
                Status::internal("Server Error")
            }
            RecallErr::Unknown(error) => {
                tracing::error!("{:?}", error);
                Status::internal("Server Error")
            }
            RecallErr::Status(status) => status,
        }),
    }
}

#[derive(Debug, thiserror::Error)]
enum RecallErr {
    #[error("database error:{0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    Unknown(#[from] anyhow::Error),
    #[error("status:{0}")]
    Status(#[from] tonic::Status),
}

impl From<DelMsgErr> for RecallErr {
    fn from(value: DelMsgErr) -> Self {
        match value {
            DelMsgErr::DbErr(db_err) => Self::Db(db_err),
            DelMsgErr::WithoutPrivilege => {
                Self::Status(Status::permission_denied("permission denied"))
            }
            DelMsgErr::NotFound => Self::Status(Status::not_found("msg not found")),
        }
    }
}

async fn recall_msg_internal(
    server: &RpcServer<impl EmailSender>,
    request: Request<RecallMsgRequest>,
) -> Result<RecallMsgResponse, RecallErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    // delete from the database first
    del_msg(req.msg_id, Some(id), &server.db.db_pool).await?;
    Ok(RecallMsgResponse {})
}
