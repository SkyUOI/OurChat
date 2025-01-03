use crate::{component::EmailSender, db, server::RpcServer};
use base::time::to_google_timestamp;
use pb::ourchat::session::get_session_info::v1::{
    GetSessionInfoRequest, GetSessionInfoResponse, QueryValues,
};
use tonic::{Request, Response, Status};

use super::query_session;

pub async fn get_session_info(
    server: &RpcServer<impl EmailSender>,
    request: Request<GetSessionInfoRequest>,
) -> Result<Response<GetSessionInfoResponse>, Status> {
    match get_session_info_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            let status = match e {
                GetSessionErr::Db(e) => {
                    tracing::error!("Database error: {}", e);
                    Status::internal(e.to_string())
                }
                GetSessionErr::Status(s) => s,
                GetSessionErr::Internal(e) => {
                    tracing::error!("Internal error: {}", e);
                    Status::internal(e.to_string())
                }
            };
            Err(status)
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum GetSessionErr {
    #[error("database error:{0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0}")]
    Status(#[from] tonic::Status),
    #[error("internal error:{0}")]
    Internal(#[from] anyhow::Error),
}

async fn get_session_info_impl(
    server: &RpcServer<impl EmailSender>,
    request: Request<GetSessionInfoRequest>,
) -> Result<GetSessionInfoResponse, GetSessionErr> {
    let mut res = GetSessionInfoResponse::default();
    let req_inner = request.into_inner();
    let session_id = req_inner.session_id.into();
    let session_data = match query_session(session_id, &server.db.db_pool).await? {
        Some(d) => d,
        None => {
            return Err(GetSessionErr::Status(Status::not_found(
                "Session not found",
            )));
        }
    };
    for i in req_inner.query_values {
        let i = match QueryValues::try_from(i) {
            Ok(i) => i,
            Err(_) => {
                return Err(GetSessionErr::Status(Status::invalid_argument(
                    "Invalid query value",
                )));
            }
        };
        match i {
            QueryValues::Unspecified => {
                tracing::warn!("Meet a unspecified request value");
            }
            QueryValues::SessionId => {
                res.session_id = Some(req_inner.session_id);
            }
            QueryValues::Name => {
                res.name = Some(session_data.name.clone());
            }
            QueryValues::AvatarKey => {
                res.avatar_key = Some(session_data.avatar_key.clone().unwrap_or_default());
            }
            QueryValues::CreatedTime => {
                res.created_time = Some(to_google_timestamp(session_data.created_time.into()));
            }
            QueryValues::UpdatedTime => {
                res.updated_time = Some(to_google_timestamp(session_data.updated_time.into()));
            }
            QueryValues::Members => {
                res.members = db::session::get_members(session_id, &server.db.db_pool)
                    .await?
                    .into_iter()
                    .map(|i| i.user_id as u64)
                    .collect();
            }
            QueryValues::OwnerId => todo!(),
            QueryValues::Size => {
                res.size = Some(session_data.size as u64);
            }
        }
    }
    Ok(res)
}
