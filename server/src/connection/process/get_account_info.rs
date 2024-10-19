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
use sea_orm::{DatabaseConnection, EntityTrait};
use std::cmp::PartialEq;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
enum Privilege {
    Stranger,
    Owner,
}

pub async fn get_account_info(
    id: Option<&UserInfo>,
    net_sender: impl NetSender,
    request_data: requests::GetAccountInfoRequest,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let privilege = match id {
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
    let id = get_id(&request_data.ocid, db_pool).await?;
    let queried_user = get_account_info_db(id, db_pool.db_pool.clone()).await?;
    let mut data_map = HashMap::new();

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
                crate::client::basic::RequestValues::DisplayName => todo!(),
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
                crate::client::basic::RequestValues::Friends => todo!(),
                crate::client::basic::RequestValues::UserName => todo!(),
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
    db_conn: DatabaseConnection,
) -> anyhow::Result<data_wrapper::User> {
    use entities::prelude::*;
    use entities::user;
    let queried_user = user::Entity::find_by_id(id).one(&db_conn).await?.unwrap();
    Ok(queried_user.into())
}

#[db_compatibility]
async fn get_friends(id: ID) -> anyhow::Result<Vec<data_wrapper::User>> {
    use entities::friend;
    use entities::prelude::*;
    todo!()
}
