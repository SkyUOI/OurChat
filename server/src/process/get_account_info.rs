use super::error_msg::{PERMISSION_DENIED, REQUEST_INVALID_VALUE, not_found};
use super::mapped_to_user_defined_status;
use crate::db;
use crate::db::session::get_all_session_relations;
use crate::process::error_msg::SERVER_ERROR;
use crate::server::RpcServer;
use anyhow::Context;
use base::consts::ID;
use deadpool_redis::redis::AsyncCommands;
use pb::service::ourchat::get_account_info::v1::{
    GetAccountInfoRequest, GetAccountInfoResponse, OWNER_PRIVILEGE, QueryValues,
};
use pb::time::to_google_timestamp;
use std::cmp::PartialEq;
use std::sync::OnceLock;
use tonic::Request;

#[derive(PartialEq, Copy, Clone)]
enum Privilege {
    Stranger,
    Owner,
}

#[derive(Debug, thiserror::Error)]
enum GetInfoError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("not found")]
    NotFound,
    #[error("status error:{0:?}")]
    StatusError(#[from] tonic::Status),
    #[error("internal error:{0:?}")]
    InternalError(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
}

async fn get_account_info_impl(
    server: &RpcServer,
    id: ID,
    request: Request<GetAccountInfoRequest>,
) -> Result<GetAccountInfoResponse, GetInfoError> {
    let request = request.into_inner();
    // query in the database
    // get id first
    let request_id = match request.id {
        Some(id) => ID(id),
        None => id,
    };
    let privilege = if id == request_id {
        Privilege::Owner
    } else {
        Privilege::Stranger
    };

    let queried_user = match db::user::get_account_info_db(request_id, &server.db.db_pool).await? {
        Some(user) => user,
        None => return Err(GetInfoError::NotFound),
    };
    let data_cell = OnceLock::new();
    let friends = async || {
        if data_cell.get().is_none() {
            let list = db::user::get_friends(request_id, &server.db.db_pool).await?;
            data_cell.set(list).unwrap();
        }
        anyhow::Ok(data_cell.get().unwrap())
    };
    let mut ret = GetAccountInfoResponse::default();

    for i in &request.request_values {
        let i = match QueryValues::try_from(*i) {
            Ok(i) => i,
            Err(_) => {
                return Err(tonic::Status::invalid_argument(REQUEST_INVALID_VALUE))?;
            }
        };
        if privilege != Privilege::Owner && OWNER_PRIVILEGE.contains(&i) {
            // cannot get the info which is owner privilege
            return Err(GetInfoError::PermissionDenied);
        } else {
            // can access the info,get from the database
            match i {
                QueryValues::Ocid => ret.ocid = Some(queried_user.ocid.clone()),
                QueryValues::Email => ret.email = Some(queried_user.email.clone()),
                QueryValues::DisplayName => {
                    if let Privilege::Owner = privilege {
                        // invalid for the owner, ignore
                    } else {
                        let friend =
                            db::user::query_contact_user_info(id, request_id, &server.db.db_pool)
                                .await?;
                        ret.display_name = Some(
                            if let Some(friend) = friend
                                && let Some(name) = friend.display_name
                            {
                                name
                            } else {
                                String::default()
                            },
                        );
                    }
                }
                QueryValues::Status => {
                    // ret.status = Some(queried_user.status.clone().unwrap_or_default());
                    let mut redis_conn = server
                        .db
                        .redis_pool
                        .get()
                        .await
                        .context("Cannot get redis' connection")?;
                    ret.status = redis_conn
                        .get(mapped_to_user_defined_status(queried_user.id))
                        .await
                        .context("Cannot get redis' information")?;
                }
                QueryValues::AvatarKey => {
                    ret.avatar_key = Some(queried_user.avatar.clone().unwrap_or_default());
                }
                QueryValues::RegisterTime => {
                    ret.register_time = Some(to_google_timestamp(queried_user.time.into()))
                }
                QueryValues::PublicUpdatedTime => {
                    ret.public_updated_time =
                        Some(to_google_timestamp(queried_user.public_update_time.into()))
                }
                QueryValues::UpdatedTime => {
                    // only owner can get
                    if privilege != Privilege::Owner {
                        return Err(GetInfoError::PermissionDenied)?;
                    }
                    ret.updated_time = Some(to_google_timestamp(queried_user.update_time.into()))
                }
                QueryValues::Sessions => {
                    // only owner can get
                    if privilege != Privilege::Owner {
                        return Err(GetInfoError::PermissionDenied)?;
                    }
                    let sessions = get_all_session_relations(id, &server.db.db_pool).await?;
                    let ids = sessions.into_iter().map(|x| x.session_id as u64).collect();
                    ret.sessions = ids;
                }
                QueryValues::Friends => {
                    let friends = friends().await?;
                    let mut ids = vec![];
                    for i in friends {
                        ids.push(i.friend_id as u64);
                    }
                    ret.friends = ids
                }
                QueryValues::UserName => ret.user_name = Some(queried_user.name.clone()),
                QueryValues::Unspecified => {}
            }
        }
    }
    Ok(ret)
}

pub async fn get_account_info(
    server: &RpcServer,
    id: ID,
    request: Request<GetAccountInfoRequest>,
) -> Result<tonic::Response<GetAccountInfoResponse>, tonic::Status> {
    match get_account_info_impl(server, id, request).await {
        Ok(d) => Ok(tonic::Response::new(d)),
        Err(e) => match e {
            GetInfoError::DbError(_) | GetInfoError::InternalError(_) => {
                tracing::error!("{}", e);
                Err(tonic::Status::internal(SERVER_ERROR))
            }
            GetInfoError::NotFound => Err(tonic::Status::not_found(not_found::USER)),
            GetInfoError::PermissionDenied => {
                Err(tonic::Status::permission_denied(PERMISSION_DENIED))
            }
            GetInfoError::StatusError(status) => Err(status),
        },
    }
}
