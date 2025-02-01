//! Functions process the requests from clients
//!
//! For grpc development, a template of unary calling is provided as follows:
//! ```ignore
//! use crate::{process::{error_msg::SERVER_ERROR}, server::RpcServer};
//! use pb::service::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
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
//!                 Err(Status::internal(SERVER_ERROR))
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
//!     Status(#[from] Status),
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
pub mod error_msg;
mod friends;
pub mod get_account_info;
mod message;
pub mod register;
mod session;
mod set_account_info;
pub mod unregister;
mod upload;
pub mod verify;

use base::consts::SessionID;
use deadpool_lapin::lapin::options::BasicPublishOptions;
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
pub use friends::{
    accept_friend::accept_friend, add_friend::add_friend, set_friend_info::set_friend_info,
};
pub use message::{fetch_user_msg::fetch_user_msg, recall::recall_msg, send_msg::send_msg};
pub use session::{
    accept_join_in_session::accept_join_in_session,
    accept_session::accept_session,
    add_role::add_role,
    ban::{ban_user, unban_user},
    delete_session::delete_session,
    get_session_info::get_session_info,
    join_in_session::join_in_session,
    leave_session::leave_session,
    mute::{mute_user, unmute_user},
    new_session::new_session,
    set_role::set_role,
    set_session_info::set_session_info,
};
pub use set_account_info::set_account_info;
pub use unregister::unregister;
pub use upload::upload;

use crate::SERVER_INFO;
use crate::db::session::get_members;
use crate::rabbitmq::USER_MSG_EXCHANGE;
use crate::rabbitmq::generate_route_key;
use base::consts::ID;
use entities::prelude::*;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use prost::Message;

pub mod db {
    pub use super::basic::get_id;
    pub use super::session::new_session::create_session_db;
    pub use crate::db::session::batch_join_in_session;
    pub use crate::db::session::join_in_session;
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

pub async fn check_user_exist(
    id: ID,
    db_conn: &impl ConnectionTrait,
) -> Result<bool, sea_orm::DbErr> {
    Ok(User::find()
        .filter(entities::user::Column::Id.eq(id))
        .one(db_conn)
        .await?
        .is_some())
}

enum Dest {
    User(ID),
    Session(SessionID),
}

async fn transmit_msg(
    msg: FetchMsgsResponse,
    dest: Dest,
    rabbitmq_connection: &mut deadpool_lapin::lapin::Channel,
    db_connection: &impl ConnectionTrait,
) -> anyhow::Result<()> {
    let mut buf = bytes::BytesMut::new();
    msg.encode(&mut buf)?;
    match dest {
        Dest::User(id) => {
            rabbitmq_connection
                .basic_publish(
                    USER_MSG_EXCHANGE,
                    &generate_route_key(id),
                    BasicPublishOptions::default(),
                    buf.as_ref(),
                    Default::default(),
                )
                .await?;
        }
        Dest::Session(id) => {
            for i in get_members(id, db_connection).await? {
                let dest_id = i.user_id.into();
                rabbitmq_connection
                    .basic_publish(
                        USER_MSG_EXCHANGE,
                        &generate_route_key(dest_id),
                        BasicPublishOptions::default(),
                        buf.as_ref(),
                        Default::default(),
                    )
                    .await?;
            }
        }
    }
    Ok(())
}
