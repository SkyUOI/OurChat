use super::basic::get_ocid;
use crate::component::EmailSender;
use crate::consts::ID;
use crate::db;
use crate::db::session::get_all_session_relations;
use crate::server::RpcServer;
use base::time::to_google_timestamp;
use pb::ourchat::get_account_info::v1::{
    GetAccountInfoRequest, GetAccountInfoResponse, OWNER_PRIVILEGE, RequestValues,
};
use std::cmp::PartialEq;
use std::sync::OnceLock;
use tonic::Request;

use super::get_id_from_req;

#[derive(PartialEq, Copy, Clone)]
enum Privilege {
    Stranger,
    Owner,
}

#[derive(Debug, thiserror::Error)]
enum GetInfoError {
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("not found")]
    NotFound,
    #[error("status error:{0}")]
    StatusError(#[from] tonic::Status),
    #[error("internal error:{0}")]
    InternalError(#[from] anyhow::Error),
}

async fn get_info_impl(
    server: &RpcServer<impl EmailSender>,
    request: Request<GetAccountInfoRequest>,
) -> Result<GetAccountInfoResponse, GetInfoError> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    // query in database
    // get id first
    let requests_id = match request.id {
        Some(id) => ID(id),
        None => id,
    };
    let privilege = if id == requests_id {
        Privilege::Owner
    } else {
        Privilege::Stranger
    };

    let queried_user = match db::user::get_account_info_db(requests_id, &server.db.db_pool).await? {
        Some(user) => user,
        None => return Err(GetInfoError::NotFound),
    };
    let data_cell = OnceLock::new();
    let friends = async || {
        if data_cell.get().is_none() {
            let list = db::user::get_friends(requests_id, &server.db.db_pool).await?;
            data_cell.set(list).unwrap();
        }
        anyhow::Ok(data_cell.get().unwrap())
    };
    let mut ret = GetAccountInfoResponse::default();

    for i in &request.request_values {
        let i = match RequestValues::try_from(*i) {
            Ok(i) => i,
            Err(_) => {
                return Err(tonic::Status::invalid_argument(
                    "value requetsed is invalid",
                ))?;
            }
        };
        if privilege != Privilege::Owner && OWNER_PRIVILEGE.contains(&i) {
            // cannot get the info which is owner privilege
            return Err(tonic::Status::permission_denied("permission denied"))?;
        } else {
            // can access the info,get from the database
            match i {
                RequestValues::Ocid => ret.ocid = Some(get_ocid(id, &server.db).await?),
                RequestValues::Email => ret.email = Some(queried_user.email.clone()),
                RequestValues::DisplayName => {
                    if let Privilege::Owner = privilege {
                        // invalid for owner, ignore
                    } else {
                        let friend =
                            db::user::get_one_friend(id, requests_id, &server.db.db_pool).await?;
                        ret.display_name = friend.map(|x| x.display_name);
                    }
                }
                RequestValues::Status => {
                    ret.status = Some(queried_user.status.clone().unwrap_or_default());
                }
                RequestValues::AvatarKey => {
                    ret.avatar_key = Some(queried_user.avatar.clone().unwrap_or_default());
                }
                RequestValues::RegisterTime => {
                    ret.register_time = Some(to_google_timestamp(queried_user.time.into()))
                }
                RequestValues::PublicUpdateTime => {
                    ret.public_update_time =
                        Some(to_google_timestamp(queried_user.public_update_time.into()))
                }
                RequestValues::UpdateTime => {
                    // only owner can get
                    if privilege != Privilege::Owner {
                        return Err(tonic::Status::permission_denied("permission denied"))?;
                    }
                    ret.update_time = Some(to_google_timestamp(queried_user.update_time.into()))
                }
                RequestValues::Sessions => {
                    // only owner can get
                    if privilege != Privilege::Owner {
                        return Err(tonic::Status::permission_denied("permission denied"))?;
                    }
                    let sessions = get_all_session_relations(id, &server.db.db_pool).await?;
                    let ids = sessions.into_iter().map(|x| x.session_id as u64).collect();
                    ret.sessions = ids;
                }
                RequestValues::Friends => {
                    let friends = friends().await?;
                    let mut ids = vec![];
                    for i in friends {
                        ids.push(i.friend_id as u64);
                    }
                    ret.friends = ids
                }
                RequestValues::UserName => ret.user_name = Some(queried_user.name.clone()),
                RequestValues::Unspecified => {
                    tracing::warn!("Meet a unspecified request value");
                }
            }
        }
    }
    Ok(ret)
}

pub async fn get_info<T: EmailSender>(
    server: &RpcServer<T>,
    request: Request<GetAccountInfoRequest>,
) -> Result<tonic::Response<GetAccountInfoResponse>, tonic::Status> {
    match get_info_impl(server, request).await {
        Ok(d) => Ok(tonic::Response::new(d)),
        Err(e) => match e {
            GetInfoError::DbError(db_err) => {
                tracing::error!("{}", db_err);
                Err(tonic::Status::internal("Server error"))
            }
            GetInfoError::NotFound => Err(tonic::Status::not_found("User not found")),
            GetInfoError::StatusError(status) => Err(status),
            GetInfoError::InternalError(error) => {
                tracing::error!("{}", error);
                Err(tonic::Status::internal("Server error"))
            }
        },
    }
}
