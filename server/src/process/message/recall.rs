use crate::{
    db::messages::{MsgError, del_msg},
    process::{
        error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found},
        get_id_from_req,
    },
    server::RpcServer,
};
use pb::ourchat::msg_delivery::recall::v1::{RecallMsgRequest, RecallMsgResponse};
use tonic::{Request, Response, Status};

pub async fn recall_msg(
    server: &RpcServer,
    request: Request<RecallMsgRequest>,
) -> Result<Response<RecallMsgResponse>, Status> {
    match recall_msg_internal(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(match e {
            RecallErr::Db(_) | RecallErr::Unknown(_) => {
                tracing::error!("{}", e);
                Status::internal(SERVER_ERROR)
            }
            RecallErr::Status(status) => status,
        }),
    }
}

#[derive(Debug, thiserror::Error)]
enum RecallErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("status:{0:?}")]
    Status(#[from] tonic::Status),
}

impl From<MsgError> for RecallErr {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::DbError(db_err) => Self::Db(db_err),
            MsgError::WithoutPrivilege => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            MsgError::NotFound => Self::Status(Status::not_found(not_found::MSG)),
            MsgError::UnknownError(error) => Self::Unknown(error),
        }
    }
}

async fn recall_msg_internal(
    server: &RpcServer,
    request: Request<RecallMsgRequest>,
) -> Result<RecallMsgResponse, RecallErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    // delete from the database first
    del_msg(req.msg_id, Some(id), &server.db.db_pool).await?;
    Ok(RecallMsgResponse {})
}
