use crate::process::Dest;
use crate::{
    db,
    db::messages::{MsgError, del_msg},
    process::{
        error_msg::{PERMISSION_DENIED, RECALL_TIME_LIMIT_EXCEEDED, SERVER_ERROR, not_found},
        transmit_msg,
    },
    server::RpcServer,
};
use anyhow::Context;
use base::constants::{ID, SessionID};
use pb::service::ourchat::msg_delivery::recall::v1::{
    RecallMsgRequest, RecallMsgResponse, RecallNotification,
};
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use std::time::Duration;
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
pub(super) enum RecallErr {
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
            MsgError::TimeLimitExceeded => {
                Self::Status(Status::out_of_range(RECALL_TIME_LIMIT_EXCEEDED))
            }
            MsgError::NotFound => Self::Status(Status::not_found(not_found::MSG)),
            MsgError::UnknownError(error) => Self::Unknown(error),
            MsgError::SerdeError(error) => Self::Unknown(error.into()),
        }
    }
}

async fn recall_msg_internal(
    server: &RpcServer,
    id: ID,
    request: Request<RecallMsgRequest>,
) -> Result<RecallMsgResponse, RecallErr> {
    let req = request.into_inner();
    let recall_time_limit = server.shared_data.cfg().main_cfg.recall_time_limit;
    // delete it from the database first
    del_msg(
        req.msg_id,
        req.session_id.into(),
        Some(id),
        recall_time_limit,
        &server.db.db_pool,
    )
    .await?;
    let respond_msg = RespondEventType::Recall(RecallNotification { msg_id: req.msg_id });
    let msg = db::messages::insert_msg_record(
        id.into(),
        Some(req.session_id.into()),
        respond_msg.clone(),
        false,
        &server.db.db_pool,
        false,
    )
    .await?;
    let connection = server.get_rabbitmq_manager().await?;
    let mut channel = connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    transmit_msg(
        FetchMsgsResponse {
            msg_id: msg.msg_id as u64,
            respond_event_type: Some(respond_msg),
            time: Some(msg.time.into()),
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

/// Recall a message **bypassing the permission and time-limit checks**, insert
/// the `Recall` event and broadcast it to the session.
///
/// This is the recall path taken when a recall vote passes: the majority vote
/// is treated as the authority the `RecallMsg` permission would otherwise
/// provide, so the `recall_time_limit` does not apply either.
pub(super) async fn force_recall(
    server: &RpcServer,
    session_id: SessionID,
    msg_id: u64,
    operator_id: Option<ID>,
) -> Result<(), RecallErr> {
    // delete the recalled message from the database first
    del_msg(msg_id, session_id, None, Duration::ZERO, &server.db.db_pool).await?;
    let respond_msg = RespondEventType::Recall(RecallNotification { msg_id });
    let msg = db::messages::insert_msg_record(
        operator_id,
        Some(session_id),
        respond_msg.clone(),
        false,
        &server.db.db_pool,
        false,
    )
    .await?;
    let connection = server.get_rabbitmq_manager().await?;
    let mut channel = connection
        .create_channel()
        .await
        .context("cannot create channel")?;
    transmit_msg(
        FetchMsgsResponse {
            msg_id: msg.msg_id as u64,
            respond_event_type: Some(respond_msg),
            time: Some(msg.time.into()),
        },
        Dest::Session(session_id),
        &mut channel,
        &server.db.db_pool,
    )
    .await?;
    Ok(())
}
