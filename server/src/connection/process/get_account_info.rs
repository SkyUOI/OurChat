use crate::client::requests::get_account_info::OWNER_PRIVILEGE;
use crate::connection::basic::get_id;
use crate::entities::{friend, user};
use crate::{
    DbPool,
    client::{MsgConvert, requests, response::GetAccountInfoResponse},
    connection::{NetSender, UserInfo},
    consts::ID,
    entities::prelude::*,
};
use anyhow::Context;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(PartialEq, Copy, Clone)]
enum Privilege {
    Stranger,
    Owner,
}

pub async fn get_account_info(
    user_info: Option<&UserInfo>,
    net_sender: impl NetSender,
    request_data: requests::GetAccountInfoRequest,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let privilege = match user_info {
        Some(id) => {
            if request_data.ocid == id.ocid {
                Privilege::Owner
            } else {
                Privilege::Stranger
            }
        }
        None => Privilege::Stranger,
    };
    // query in database
    // get id first
    let requests_id = get_id(&request_data.ocid, db_pool).await?;
    let queried_user = get_account_info_db(requests_id, &db_pool.db_pool).await?;
    let mut data_map = HashMap::new();
    let data_cell: OnceLock<Vec<friend::Model>> = OnceLock::new();
    let friends = || async {
        if data_cell.get().is_none() {
            let list = get_friends(requests_id, &db_pool.db_pool).await?;
            data_cell.set(list).unwrap();
        }
        anyhow::Ok(data_cell.get().unwrap())
    };

    for i in &request_data.request_values {
        if privilege != Privilege::Owner && OWNER_PRIVILEGE.contains(i) {
            // cannot get the info which is owner privilege
            data_map.insert(*i, serde_json::Value::Null);
        } else {
            // can access the info,get from the database
            data_map.insert(*i, match i {
                crate::client::basic::GetAccountValues::Ocid => {
                    serde_json::Value::String(request_data.ocid.clone())
                }
                crate::client::basic::GetAccountValues::Email => {
                    serde_json::Value::String(queried_user.email.clone())
                }
                crate::client::basic::GetAccountValues::DisplayName => {
                    // TODO:optimize the performance
                    match user_info {
                        None => serde_json::Value::Null,
                        Some(user_info) => {
                            let friend =
                                get_one_friend(user_info.id, requests_id, &db_pool.db_pool)
                                    .await?
                                    .unwrap();
                            serde_json::Value::String(friend.display_name)
                        }
                    }
                }
                crate::client::basic::GetAccountValues::Status => todo!(),
                crate::client::basic::GetAccountValues::AvatarKey => todo!(),
                crate::client::basic::GetAccountValues::Time => {
                    serde_json::Value::String(queried_user.time.to_rfc3339())
                }
                crate::client::basic::GetAccountValues::PublicUpdateTime => {
                    serde_json::Value::String(queried_user.public_update_time.to_rfc3339())
                }
                crate::client::basic::GetAccountValues::UpdateTime => {
                    serde_json::Value::String(queried_user.update_time.to_rfc3339())
                }
                crate::client::basic::GetAccountValues::Sessions => todo!(),
                crate::client::basic::GetAccountValues::Friends => {
                    // TODO:optimize the performance
                    let friends = friends().await?;
                    let mut ret = vec![];
                    for i in friends {
                        ret.push(serde_json::Value::String(
                            get_account_info_db(i.friend_id.into(), &db_pool.db_pool)
                                .await?
                                .ocid,
                        ));
                    }
                    serde_json::Value::Array(ret)
                }
                crate::client::basic::GetAccountValues::UserName => {
                    serde_json::Value::String(queried_user.name.clone())
                }
            });
        }
    }

    let response = GetAccountInfoResponse::success(data_map);
    net_sender.send(response.to_msg()).await?;
    Ok(())
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
