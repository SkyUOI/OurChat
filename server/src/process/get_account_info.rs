use super::basic::get_id;
use crate::component::EmailSender;
use crate::entities::{friend, user};
use crate::pb::ourchat::get_account_info::v1::{
    GetAccountInfoRequest, GetAccountInfoResponse, OWNER_PRIVILEGE, RequestValues,
};
use crate::server::RpcServer;
use crate::utils::to_google_timestamp;
use crate::{consts::ID, entities::prelude::*};
use anyhow::Context;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::cmp::PartialEq;
use std::sync::OnceLock;
use tonic::Request;

use super::get_id_from_req;

#[derive(PartialEq, Copy, Clone)]
enum Privilege {
    Stranger,
    Owner,
}

pub async fn get_info<T: EmailSender>(
    server: &RpcServer<T>,
    request: Request<GetAccountInfoRequest>,
) -> Result<tonic::Response<GetAccountInfoResponse>, tonic::Status> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    // query in database
    // get id first
    let requests_id = match get_id(&request.ocid, &server.db).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Database error:{e}");
            return Err(tonic::Status::internal("database error"));
        }
    };
    let privilege = if id == requests_id {
        Privilege::Owner
    } else {
        Privilege::Stranger
    };

    let queried_user = match get_account_info_db(requests_id, &server.db.db_pool).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Database error:{e}");
            return Err(tonic::Status::internal("database error"));
        }
    };
    let data_cell = OnceLock::new();
    let friends = || async {
        if data_cell.get().is_none() {
            let list = get_friends(requests_id, &server.db.db_pool).await?;
            data_cell.set(list).unwrap();
        }
        anyhow::Ok(data_cell.get().unwrap())
    };
    let mut ret = GetAccountInfoResponse::default();

    for i in &request.request_values {
        let i = RequestValues::try_from(*i).unwrap();
        if privilege != Privilege::Owner && OWNER_PRIVILEGE.contains(&i) {
            // cannot get the info which is owner privilege
            return Err(tonic::Status::permission_denied("permission denied"));
        } else {
            // can access the info,get from the database
            match i {
                RequestValues::Ocid => ret.ocid = Some(request.ocid.clone()),
                RequestValues::Email => ret.email = Some(queried_user.email.clone()),
                RequestValues::DisplayName => {
                    // TODO:optimize the performance
                    if let Privilege::Owner = privilege {
                    } else {
                        let friend =
                            match get_one_friend(id, requests_id, &server.db.db_pool).await {
                                Ok(friend) => friend,
                                Err(e) => {
                                    tracing::error!("Database error:{e}");
                                    return Err(tonic::Status::internal("database error"));
                                }
                            }
                            .unwrap();
                        ret.display_name = Some(friend.display_name)
                    }
                }
                RequestValues::Status => todo!(),
                RequestValues::AvatarKey => todo!(),
                RequestValues::RegisterTime => {
                    ret.register_time = Some(to_google_timestamp(queried_user.time.into()))
                }
                RequestValues::PublicUpdateTime => {
                    ret.public_update_time =
                        Some(to_google_timestamp(queried_user.public_update_time.into()))
                }
                RequestValues::UpdateTime => {
                    ret.update_time = Some(to_google_timestamp(queried_user.update_time.into()))
                }
                RequestValues::Sessions => todo!(),
                RequestValues::Friends => {
                    // TODO:optimize the performance
                    let friends = match friends().await {
                        Ok(friends) => friends,
                        Err(e) => {
                            tracing::error!("Database error:{e}");
                            return Err(tonic::Status::internal("database error"));
                        }
                    };
                    let mut ids = vec![];
                    for i in friends {
                        ids.push(
                            match get_account_info_db(i.friend_id.into(), &server.db.db_pool).await
                            {
                                Ok(user) => user,
                                Err(e) => {
                                    tracing::error!("Database error:{e}");
                                    return Err(tonic::Status::internal("database error"));
                                }
                            }
                            .ocid,
                        );
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

    Ok(tonic::Response::new(ret))
}

async fn get_account_info_db(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<user::Model> {
    let queried_user = User::find_by_id(id).one(db_conn).await?.unwrap();
    Ok(queried_user)
}

async fn get_friends(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<Vec<friend::Model>> {
    let id: u64 = id.into();
    let friends = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .all(db_conn)
        .await?;
    Ok(friends)
}

async fn get_one_friend(
    id: ID,
    friend_id: ID,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<Option<friend::Model>> {
    let id: u64 = id.into();
    let friend_id: u64 = friend_id.into();
    let friend = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .filter(friend::Column::FriendId.eq(friend_id))
        .one(db_conn)
        .await
        .with_context(|| {
            format!(
                "Failed to get the friend of user {} and friend {}",
                id, friend_id
            )
        })?;
    Ok(friend)
}
