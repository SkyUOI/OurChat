//! Functions process the requests from clients

// # Template for developers
// ```
// use crate::{
//     DbPool,
//     client::requests::UserSendMsgRequest,
//     connection::{NetSender, UserInfo},
//     consts::ID,
// };
// use derive::db_compatibility;
// use sea_orm::DatabaseConnection;
//
// pub async fn xxx(
//     user_info: &UserInfo,
//     request: UserSendMsgRequest,
//     net_sender: impl NetSender,
//     db_pool: &DbPool,
// ) -> anyhow::Result<()> {
//     let ret = match db_oper(user_info.id, &db_pool.db_pool).await
//     {
//         Ok(_) => todo!(),
//         Err(e) => {
//             tracing::error!("Database error:{e}");
//             todo!()
//         }
//     };
//     net_sender.send(ret.to_msg()).await?;
//     Ok(())
// }
//
// #[db_compatibility]
// async fn db_oper(
//     user_id: ID,
//     db_conn: &DatabaseConnection,
// ) -> anyhow::Result<()> {
//     use entities::prelude::*;
//     use entities::user_chat_msg;
//     Ok(())
// }
// ```

pub mod get_account_info;
mod get_user_msg;
pub mod login;
pub mod new_session;
pub mod register;
mod send_msg;
mod set_account_info;
mod set_friend_info;
pub mod unregister;
mod upload;
pub mod verify;

pub use get_user_msg::get_user_msg;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
pub use new_session::accept_session;
pub use new_session::new_session;
pub use send_msg::send_msg;
use serde::Deserialize;
use serde::Serialize;
pub use set_account_info::set_account_info;
pub use set_friend_info::set_friend_info;
use tonic::Request;
use tonic::Status;
pub use unregister::unregister;
pub use upload::upload;

use crate::SERVER_INFO;
use crate::consts::ID;

#[derive(Debug, Serialize, Deserialize)]
struct JWTdata {
    id: ID,
    exp: usize,
}

fn wrong_password() -> tonic::Status {
    Status::unauthenticated("wrong password")
}

const ACCESS_TOKEN_LEN: usize = 20;
const EXPIRE_TIME: usize = 3600 * 7;

pub fn generate_access_token(id: ID) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &JWTdata {
            id,
            exp: EXPIRE_TIME,
        },
        &EncodingKey::from_secret(SERVER_INFO.secret.as_bytes()),
    )
    .unwrap()
}

#[derive(Debug, thiserror::Error)]
enum ErrAuth {
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
    match req.metadata().get("id") {
        Some(id) => Some(ID(id.to_str().unwrap().parse::<u64>().unwrap())),
        None => None,
    }
}
