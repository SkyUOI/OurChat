use crate::{
    db,
    db::messages::MsgError,
    process::error_msg::{SERVER_ERROR, TIME_FORMAT_ERROR},
    server::RpcServer,
};
use base::constants::ID;
use pb::{
    service::ourchat::msg_delivery::v1::{
        FetchMsgsResponse, FetchSessionHistoryRequest, FetchSessionHistoryResponse,
    },
    time::TimeStampUtc,
};
use tonic::{Request, Response, Status};

pub async fn fetch_session_history(
    server: &RpcServer,
    id: ID,
    request: Request<FetchSessionHistoryRequest>,
) -> Result<Response<FetchSessionHistoryResponse>, Status> {
    match fetch_session_history_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            FetchSessionHistoryErr::Db(_) | FetchSessionHistoryErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            FetchSessionHistoryErr::Status(s) => Err(s),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum FetchSessionHistoryErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

impl From<MsgError> for FetchSessionHistoryErr {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::DbError(e) => FetchSessionHistoryErr::Db(e),
            MsgError::UnknownError(e) => FetchSessionHistoryErr::Internal(e),
            MsgError::PermissionDenied => {
                FetchSessionHistoryErr::Status(Status::permission_denied("permission denied"))
            }
            MsgError::NotFound => FetchSessionHistoryErr::Status(Status::not_found("not found")),
            MsgError::SerdeError(e) => FetchSessionHistoryErr::Internal(e.into()),
        }
    }
}

async fn fetch_session_history_impl(
    server: &RpcServer,
    id: ID,
    request: Request<FetchSessionHistoryRequest>,
) -> Result<FetchSessionHistoryResponse, FetchSessionHistoryErr> {
    let req = request.into_inner();
    let session_id = req.session_id;
    let limit = req.limit.min(100); // Cap at 100 messages per request
    let before_time: TimeStampUtc = req
        .before_time
        .ok_or_else(|| Status::invalid_argument("before_time is required"))?
        .try_into()
        .map_err(|_| Status::invalid_argument(TIME_FORMAT_ERROR))?;

    let db_conn = &server.db.db_pool;
    let session_id = base::constants::SessionID(session_id);
    // Fetch one extra to determine has_more
    let msgs = db::messages::get_session_history_msgs(
        id,
        session_id,
        before_time.into(),
        limit + 1,
        db_conn,
    )
    .await?;

    let has_more = msgs.len() > limit as usize;
    let result_msgs: Vec<FetchMsgsResponse> = msgs
        .into_iter()
        .take(limit as usize)
        .map(|msg_model| {
            let respond_event_type = serde_json::from_value(msg_model.msg_data).ok();
            FetchMsgsResponse {
                respond_event_type,
                msg_id: msg_model.msg_id as u64,
                time: Some(msg_model.time.into()),
            }
        })
        .collect();

    Ok(FetchSessionHistoryResponse {
        messages: result_msgs,
        has_more,
    })
}
