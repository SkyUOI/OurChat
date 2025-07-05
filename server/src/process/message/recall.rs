use crate::process::Dest;
use crate::{
    db,
    db::messages::{MsgError, del_msg},
    process::{
        error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found},
        transmit_msg,
    },
    server::RpcServer,
};
use anyhow::Context;
use base::consts::ID;
use pb::service::ourchat::msg_delivery::recall::v1::{
    RecallMsgRequest, RecallMsgResponse, RecallNotification,
};
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::time::to_google_timestamp;
use tonic::{Request, Response, Status};

pub async fn recall_msg(
    server: &RpcServer,
    id: ID,
    request: Request<RecallMsgRequest>,
) -> Result<Response<RecallMsgResponse>, Status> {
    match recall_msg_internal(server, id, request).await {
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
    Status(#[from] Status),
}

impl From<MsgError> for RecallErr {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::DbError(db_err) => Self::Db(db_err),
            MsgError::PermissionDenied => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            MsgError::NotFound => Self::Status(Status::not_found(not_found::MSG)),
            MsgError::UnknownError(error) => Self::Unknown(error),
        }
    }
}

async fn recall_msg_internal(
    server: &RpcServer,
    id: ID,
    request: Request<RecallMsgRequest>,
) -> Result<RecallMsgResponse, RecallErr> {
    let req = request.into_inner();
    // delete it from the database first
    del_msg(
        req.msg_id,
        req.session_id.into(),
        Some(id),
        &server.db.db_pool,
    )
    .await?;
    let respond_msg = RespondEventType::Recall(RecallNotification { msg_id: req.msg_id });
    // TODO: is_encrypted
    let msg = db::messages::insert_msg_record(
        id,
        Some(req.session_id.into()),
        respond_msg.clone(),
        false,
        &server.db.db_pool,
        false,
    )
    .await?;
    let connection = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbit connection")?;
    let mut channel = connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    transmit_msg(
        FetchMsgsResponse {
            msg_id: msg.msg_id as u64,
            respond_event_type: Some(respond_msg),
            time: Some(to_google_timestamp(msg.time.into())),
        },
        Dest::Session(req.session_id.into()),
        &mut channel,
        &server.db.db_pool,
    )
    .await?;
    Ok(RecallMsgResponse {
        msg_id: msg.msg_id as u64,
    })
}
