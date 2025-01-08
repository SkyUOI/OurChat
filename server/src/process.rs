//! Functions process the requests from clients
//!
//! For grpc development, a template of unary calling is provided as follows:
//! ```ignore
//! use crate::{component::EmailSender, server::RpcServer};
//! use pb::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
//! use tonic::{Request, Response, Status};
//!
//! pub async fn set_role(
//!     server: &RpcServer,
//!     request: Request<SetRoleRequest>,
//! ) -> Result<Response<SetRoleResponse>, Status> {
//!     match set_role_impl(server, request).await {
//!         Ok(res) => Ok(Response::new(res)),
//!         Err(e) => match e {
//!             SetRoleErr::Db(_) | SetRoleErr::Internal(_) => {
//!                 tracing::error!("{}", e);
//!                 Err(Status::internal("Server Error"))
//!             }
//!             SetRoleErr::Status(status) => Err(status),
//!         },
//!     }
//! }
//!
//! #[derive(thiserror::Error, Debug)]
//! enum SetRoleErr {
//!     #[error("database error:{0:?}")]
//!     Db(#[from] sea_orm::DbErr),
//!     #[error("status error:{0:?}")]
//!     Status(#[from] tonic::Status),
//!     #[error("internal error:{0:?}")]
//!     Internal(#[from] anyhow::Error),
//! }
//!
//! async fn set_role_impl(
//!     server: &RpcServer,
//!     request: Request<SetRoleRequest>,
//! ) -> Result<SetRoleResponse, SetRoleErr> {
//!     todo!()
//! }
//! ```

pub mod auth;
pub mod basic;
mod download;
pub mod get_account_info;
mod message;
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
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use tonic::Request;

pub use download::download;
pub use message::{fetch_user_msg::fetch_user_msg, recall::recall_msg, send_msg::send_msg};
pub use session::{
    accept_session::accept_session, add_role::add_role, get_session_info::get_session_info,
    new_session::new_session, set_role::set_role, set_session_info::set_session_info,
};
pub use set_account_info::{error_msg_consts, set_account_info};
pub use set_friend_info::set_friend_info;
pub use unregister::unregister;
pub use upload::upload;

use crate::SERVER_INFO;
use base::consts::ID;
use entities::operations;
use entities::prelude::*;

pub mod db {
    pub use super::basic::get_id;
    pub use super::session::new_session::{
        add_to_session, batch_add_to_session, create_session_db,
    };
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

async fn _get_requests(id: ID, db_conn: &impl ConnectionTrait) -> anyhow::Result<Vec<String>> {
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
