//! Functions process the requests from clients

mod accept_session;
pub mod auth;
pub mod basic;
mod download;
pub mod get_account_info;
mod message;
pub mod new_session;
pub mod register;
mod session;
mod set_account_info;
mod set_friend_info;
pub mod unregister;
mod upload;
pub mod verify;

use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use tonic::Request;

pub use accept_session::accept_session;
pub use download::download;
pub use message::{fetch_user_msg::fetch_user_msg, recall::recall_msg, send_msg::send_msg};
pub use new_session::new_session;
pub use session::{get_session_info::get_session_info, set_session_info::set_session_info};
pub use set_account_info::set_account_info;
pub use set_friend_info::set_friend_info;
pub use unregister::unregister;
pub use upload::upload;

use crate::SERVER_INFO;
use crate::consts::ID;
use entities::operations;
use entities::prelude::*;

pub mod db {
    pub use super::basic::get_id;
    pub use super::new_session::{add_to_session, batch_add_to_session, create_session_db};
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTdata {
    pub id: ID,
    exp: i64,
}

const EXPIRE_TIME: Duration = Duration::from_days(5);

pub fn generate_access_token(id: ID) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &JWTdata {
            id,
            exp: chrono::Utc::now().timestamp() + EXPIRE_TIME.as_secs() as i64,
        },
        &EncodingKey::from_secret(SERVER_INFO.secret.as_bytes()),
    )
    .unwrap()
}

pub fn check_token(token: &str) -> Option<JWTdata> {
    match access_token(token) {
        Ok(data) => {
            if chrono::offset::Utc::now().timestamp() < data.exp {
                Some(data)
            } else {
                None
            }
        }
        Err(_) => {
            tracing::trace!("jwt format wrong");
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrAuth {
    #[error("Expire")]
    Expire,
    #[error("JWT error")]
    JWT(#[from] jsonwebtoken::errors::Error),
}

pub fn access_token(token: &str) -> Result<JWTdata, ErrAuth> {
    let token = jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(SERVER_INFO.secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token.claims)
}

pub fn get_id_from_req<T>(req: &Request<T>) -> Option<ID> {
    req.metadata()
        .get("id")
        .map(|id| ID(id.to_str().unwrap().parse::<u64>().unwrap()))
}

struct UserInfo {
    id: ID,
    ocid: String,
}

async fn get_requests(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<Vec<String>> {
    let id: u64 = id.into();
    let stored_requests = Operations::find()
        .filter(operations::Column::UserId.eq(id))
        .all(db_conn)
        .await?;
    let mut ret = Vec::new();
    for i in stored_requests {
        if i.once {
            Operations::delete_by_id(i.oper_id).exec(db_conn).await?;
        }
        if i.expires_at < chrono::Utc::now() {
            Operations::delete_by_id(i.oper_id).exec(db_conn).await?;
            continue;
        }
        ret.push(i.operation);
    }
    Ok(ret)
}
