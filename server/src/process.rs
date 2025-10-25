//! Functions process the requests from clients
//!
//! For grpc development, a template of unary calling is provided as follows:
//! ```ignore
//! use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
//! use base::consts::ID;
//! use pb::service::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
//! use tonic::{Request, Response, Status};
//!
//! pub async fn set_role(
//!     server: &RpcServer,
//!     id: ID,
//!     request: Request<SetRoleRequest>,
//! ) -> Result<Response<SetRoleResponse>, Status> {
//!     match set_role_impl(server, id, request).await {
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
//!     id: ID,
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
mod server_manage;
mod session;
mod set_self_info;
pub mod unregister;
mod upload;
pub mod verify;
pub mod webrtc;

use base::consts::SessionID;
use deadpool_lapin::lapin::options::BasicPublishOptions;
use entities::message_records;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::time::Duration;
use tonic::Request;

pub use download::download;
pub use friends::{
    accept_friend_invitation::accept_friend_invitation, add_friend::add_friend,
    delete_friend::delete_friend, set_friend_info::set_friend_info,
};
pub use message::{fetch_user_msg::fetch_user_msg, recall::recall_msg, send_msg::send_msg};
pub use server_manage::{
    announcement::{
        add_announcement::add_announcement,
        get_announcement::{get_announcement_by_id, get_announcements_by_time},
        publish_announcement::publish_announcement,
    },
    delete_account::delete_account,
    set_server_status::set_server_status,
};
pub use session::{
    accept_join_session_invitation::accept_join_session_invitation,
    add_role::add_role,
    allow_user_join_session::allow_user_join_session,
    ban::{ban_user, unban_user},
    delete_session::delete_session,
    e2eeize_and_dee2eeize_session::dee2eeize_session,
    e2eeize_and_dee2eeize_session::e2eeize_session,
    get_role::get_role,
    get_session_info::get_session_info,
    invite_user_to_session::invite_user_to_session,
    join_session::join_session,
    leave_session::leave_session,
    mute::{mute_user, unmute_user},
    new_session::new_session,
    session_room_key::send_room_key,
    set_role::set_role,
    set_session_info::set_session_info,
};
pub use set_self_info::set_self_info;
pub use unregister::unregister;
pub use upload::upload;
pub use webrtc::create_room::create_room;

use crate::SERVER_INFO;
use crate::db::messages::MsgError;
use crate::db::session::get_members;
use crate::rabbitmq::USER_MSG_BROADCAST_EXCHANGE;
use crate::rabbitmq::USER_MSG_DIRECT_EXCHANGE;
use crate::rabbitmq::generate_route_key;
use base::consts::ID;
use entities::prelude::*;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use prost::Message;

pub mod db {
    pub use super::basic::get_id;
    pub use crate::db::session::batch_join_in_session;
    pub use crate::db::session::create_session_db;
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

pub fn check_token(token: &str) -> Result<JWTdata, ErrAuth> {
    let v: Vec<_> = token.split_whitespace().collect();
    if v.len() != 2 {
        return Err(ErrAuth::IncorrectFormat);
    }
    if v[0] != "Bearer" {
        return Err(ErrAuth::UnsupportedAuthorizationHeader);
    }
    let data = decode_token(v[1])?;
    if chrono::offset::Utc::now().timestamp() < data.exp {
        Ok(data)
    } else {
        Err(ErrAuth::Expire)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrAuth {
    #[error("Expire")]
    Expire,
    #[error("JWT error")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("Unsupported authorization header, only support Bearer")]
    UnsupportedAuthorizationHeader,
    #[error("Authorization: Bearer <jwt>")]
    IncorrectFormat,
}

/// Decodes a JWT token and returns the contained claims as `JWTdata`.
///
/// # Arguments
/// * `token` - A string slice that holds the JWT token to be decoded.
///
/// # Returns
/// * `Ok(JWTdata)` - The decoded claims if the token is valid.
/// * `Err(ErrAuth)` - An error if the token is invalid or the decoding process fails.
pub fn decode_token(token: &str) -> Result<JWTdata, ErrAuth> {
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

pub enum Dest {
    User(ID),
    Session(SessionID),
    All,
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
                    USER_MSG_DIRECT_EXCHANGE,
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
                        USER_MSG_DIRECT_EXCHANGE,
                        &generate_route_key(dest_id),
                        BasicPublishOptions::default(),
                        buf.as_ref(),
                        Default::default(),
                    )
                    .await?;
            }
        }
        Dest::All => {
            rabbitmq_connection
                .basic_publish(
                    USER_MSG_BROADCAST_EXCHANGE,
                    "",
                    BasicPublishOptions::default(),
                    buf.as_ref(),
                    Default::default(),
                )
                .await?;
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum MsgInsTransmitErr {
    #[error("db error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    Unknown(#[from] anyhow::Error),
    #[error("permission denied")]
    PermissionDenied,
    #[error("not found")]
    NotFound,
}

impl From<MsgError> for MsgInsTransmitErr {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::DbError(db_err) => Self::Db(db_err),
            MsgError::UnknownError(error) => Self::Unknown(error),
            MsgError::PermissionDenied => Self::PermissionDenied,
            MsgError::NotFound => Self::NotFound,
        }
    }
}

/// Insert a new message record into the database and transmit it to RabbitMQ(Corresponding user).
///
/// `sender_id` and `session_id` specify the sender and session of the message,
/// respectively. `msg` is the message content. `dest` is the destination of the
/// message. `is_encrypted` specifies whether the message is encrypted. `db_conn`
/// is the database connection. `rmq_chan` is the RabbitMQ channel.
///
/// The message record is inserted with `is_all_user` set to `false`. The
/// `time` field of the message record is set to the current time. The `msg_id`
/// field of the message record is set to the auto-incrementing ID of the
/// message record. The `msg_data` field of the message record is set to the
/// serialized `RespondEventType`.
///
/// After inserting the message record, the function transmits the message to
/// RabbitMQ using `transmit_msg`.
///
/// Returns `Ok(Model)` if the message is inserted and transmitted successfully.
/// Returns `Err(MsgInsTransmitErr)` if an error occurs.
pub async fn message_insert_and_transmit(
    sender_id: Option<ID>,
    session_id: Option<SessionID>,
    msg: RespondEventType,
    dest: Dest,
    is_encrypted: bool,
    db_conn: &impl ConnectionTrait,
    rmq_chan: &mut deadpool_lapin::lapin::Channel,
) -> Result<message_records::Model, MsgInsTransmitErr> {
    let msg_model = crate::db::messages::insert_msg_record(
        sender_id,
        session_id,
        msg.clone(),
        is_encrypted,
        db_conn,
        false,
    )
    .await?;
    let fetch_response = FetchMsgsResponse {
        msg_id: msg_model.msg_id as u64,
        time: Some(msg_model.time.into()),
        respond_event_type: Some(msg),
    };
    transmit_msg(fetch_response, dest, rmq_chan, db_conn).await?;
    Ok(msg_model)
}

fn mapped_to_user_defined_status(user_id: impl Display) -> String {
    format!("user_defined_status:{user_id}")
}
