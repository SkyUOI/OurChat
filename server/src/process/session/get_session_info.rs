use super::query_session;
use crate::db::session::in_session;
use crate::process::error_msg::{self, REQUEST_INVALID_VALUE, SERVER_ERROR, not_found};
use crate::{db, server::RpcServer};
use base::consts::ID;
use pb::service::ourchat::session::get_session_info::v1::RoleInfo;
use pb::service::ourchat::session::get_session_info::v1::{
    GetSessionInfoRequest, GetSessionInfoResponse, QueryValues,
};
use pb::time::to_google_timestamp;
use tonic::{Request, Response, Status};

pub async fn get_session_info(
    server: &RpcServer,
    id: ID,
    request: Request<GetSessionInfoRequest>,
) -> Result<Response<GetSessionInfoResponse>, Status> {
    match get_session_info_impl(server, id, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            let status = match e {
                GetSessionErr::Db(_) | GetSessionErr::Internal(_) => {
                    tracing::error!("{}", e);
                    Status::internal(SERVER_ERROR)
                }
                GetSessionErr::Status(s) => s,
            };
            Err(status)
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum GetSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn get_session_info_impl(
    server: &RpcServer,
    id: ID,
    request: Request<GetSessionInfoRequest>,
) -> Result<GetSessionInfoResponse, GetSessionErr> {
    let mut res = GetSessionInfoResponse::default();
    let req_inner = request.into_inner();
    let session_id = req_inner.session_id.into();
    let session_data = match query_session(session_id, &server.db.db_pool).await? {
        Some(d) => d,
        None => {
            return Err(GetSessionErr::Status(Status::not_found(not_found::SESSION)));
        }
    };
    let in_session = in_session(id, session_id, &server.db.db_pool).await?;
    for i in req_inner.query_values {
        let i = match QueryValues::try_from(i) {
            Ok(i) => i,
            Err(_) => {
                return Err(GetSessionErr::Status(Status::invalid_argument(
                    REQUEST_INVALID_VALUE,
                )));
            }
        };
        match i {
            QueryValues::Unspecified => {}
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
                if !in_session {
                    Err(Status::permission_denied(error_msg::PERMISSION_DENIED))?
                }
                res.members = db::session::get_members(session_id, &server.db.db_pool)
                    .await?
                    .into_iter()
                    .map(|i| i.user_id as u64)
                    .collect();
            }
            QueryValues::Roles => {
                if !in_session {
                    Err(Status::permission_denied(error_msg::PERMISSION_DENIED))?
                }
                let all_users =
                    db::session::get_all_roles_of_session(session_id, &server.db.db_pool).await?;
                res.roles = all_users
                    .into_iter()
                    .map(|i| RoleInfo {
                        user_id: i.user_id as u64,
                        role: i.role_id,
                    })
                    .collect();
            }
            QueryValues::Size => {
                res.size = Some(session_data.size as u64);
            }
            QueryValues::Description => {
                res.description = Some(session_data.description.clone());
            }
        }
    }
    Ok(res)
}
