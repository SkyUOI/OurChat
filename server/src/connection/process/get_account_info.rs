use crate::client::requests::get_account_info::OWNER_PRIVILEGE;
use crate::connection::basic::get_id;
use crate::db::data_wrapper;
use crate::{
    DbPool,
    client::{MsgConvert, requests, response::GetAccountInfoResponse},
    connection::{NetSender, UserInfo},
    consts::ID,
};
use derive::db_compatibility;
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
    let data_cell: OnceLock<Vec<data_wrapper::Friend>> = OnceLock::new();
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
                crate::client::basic::RequestValues::Ocid => {
                    serde_json::Value::String(request_data.ocid.clone())
                }
                crate::client::basic::RequestValues::Email => {
                    serde_json::Value::String(queried_user.email.clone())
                }
                crate::client::basic::RequestValues::DisplayName => {
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
                crate::client::basic::RequestValues::Status => todo!(),
                crate::client::basic::RequestValues::AvatarKey => todo!(),
                crate::client::basic::RequestValues::Time => {
                    serde_json::Value::String(queried_user.time.to_rfc3339())
                }
                crate::client::basic::RequestValues::PublicUpdateTime => {
                    serde_json::Value::String(queried_user.public_update_time.to_rfc3339())
                }
                crate::client::basic::RequestValues::UpdateTime => {
                    serde_json::Value::String(queried_user.update_time.to_rfc3339())
                }
                crate::client::basic::RequestValues::Sessions => todo!(),
                crate::client::basic::RequestValues::Friends => {
                    // TODO:optimize the performance
                    let friends = friends().await?;
                    let mut ret = vec![];
                    for i in friends {
                        ret.push(serde_json::Value::String(
                            get_account_info_db(i.friend_id, &db_pool.db_pool)
                                .await?
                                .ocid,
                        ));
                    }
                    serde_json::Value::Array(ret)
                }
                crate::client::basic::RequestValues::UserName => {
                    serde_json::Value::String(queried_user.name.clone())
                }
            });
        }
    }

    let response = GetAccountInfoResponse::success(data_map);
    net_sender.send(response.to_msg()).await?;
    Ok(())
}

#[db_compatibility]
async fn get_account_info_db(
    id: ID,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<data_wrapper::User> {
    use entities::prelude::*;
    use entities::user;
    let queried_user = user::Entity::find_by_id(id).one(db_conn).await?.unwrap();
    Ok(queried_user.into())
}

#[db_compatibility]
async fn get_friends(
    id: ID,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<Vec<data_wrapper::Friend>> {
    use entities::friend;
    use entities::prelude::*;

    let id: u64 = id.into();
    let friends = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .all(db_conn)
        .await?;
    Ok(friends.into_iter().map(|d| d.into()).collect())
}

#[db_compatibility]
async fn get_one_friend(
    id: ID,
    friend_id: ID,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<Option<data_wrapper::Friend>> {
    use entities::friend;
    use entities::prelude::*;

    let id: u64 = id.into();
    let friend_id: u64 = friend_id.into();
    let friend = Friend::find()
        .filter(friend::Column::UserId.eq(id))
        .filter(friend::Column::FriendId.eq(friend_id))
        .one(db_conn)
        .await?;
    Ok(friend.map(|d| d.into()))
}
