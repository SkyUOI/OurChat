use super::super::get_id_from_req;
use crate::{
    db::{self, messages::MsgError, session::check_user_in_session},
    process::error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found},
    server::RpcServer,
};
use base::consts::ID;
use pb::ourchat::msg_delivery::v1::{SendMsgRequest, SendMsgResponse};
use tonic::{Request, Response, Status};

pub async fn send_msg(
    server: &RpcServer,
    request: Request<SendMsgRequest>,
) -> Result<Response<SendMsgResponse>, Status> {
    match send_msg_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => Err(match e {
            SendMsgErr::Db(_) | SendMsgErr::Internal(_) => {
                tracing::error!("{}", e);
                Status::internal(SERVER_ERROR)
            }
            SendMsgErr::Status(status) => status,
        }),
    }
}

#[derive(thiserror::Error, Debug)]
enum SendMsgErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

impl From<MsgError> for SendMsgErr {
    fn from(e: MsgError) -> Self {
        match e {
            MsgError::DbError(e) => Self::Db(e),
            MsgError::UnknownError(error) => Self::Internal(error),
            MsgError::WithoutPrivilege => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            MsgError::NotFound => Self::Status(Status::not_found(not_found::MSG)),
        }
    }
}

async fn send_msg_impl(
    server: &RpcServer,
    request: Request<SendMsgRequest>,
) -> Result<SendMsgResponse, SendMsgErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let db_conn = server.db.clone();
    // check
    if check_user_in_session(id, req.session_id.into(), &db_conn.db_pool).await? {
        Err(Status::permission_denied(PERMISSION_DENIED))?;
    }
    let msg_id = db::messages::insert_msg_record(
        id,
        ID(req.session_id),
        serde_json::value::to_value(req.bundle_msgs).unwrap(),
        req.is_encrypted,
        &db_conn.db_pool,
    )
    .await?;
    Ok(SendMsgResponse {
        msg_id: msg_id.into(),
    })
}
