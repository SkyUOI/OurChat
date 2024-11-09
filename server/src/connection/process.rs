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

mod get_account_info;
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

pub use get_account_info::get_account_info;
pub use get_user_msg::get_user_msg;
pub use new_session::accept_session;
pub use new_session::new_session;
use redis::AsyncCommands;
pub use send_msg::send_msg;
pub use set_account_info::set_account_info;
pub use set_friend_info::set_friend_info;
use tonic::Status;
pub use unregister::unregister;
pub use upload::upload;

use crate::utils;

fn wrong_password() -> tonic::Status {
    Status::unauthenticated("wrong password")
}

const ACCESS_TOKEN_LEN: usize = 20;
const EXPIRE_TIME: u64 = 3600 * 7;

pub fn generate_access_token() -> String {
    utils::generate_random_string(ACCESS_TOKEN_LEN)
}

pub fn access_token_redis(name: &str) -> String {
    format!("access_token:{}", name)
}

pub async fn put_access_token(
    redis_conn: &deadpool_redis::Pool,
    user: &str,
    token: &str,
) -> anyhow::Result<()> {
    let mut conn = redis_conn.get().await?;
    let _: () = conn
        .set_ex(access_token_redis(user), token, EXPIRE_TIME)
        .await?;
    Ok(())
}

pub async fn get_new_access_token(
    redis_conn: &deadpool_redis::Pool,
    user: &str,
) -> anyhow::Result<String> {
    let token = generate_access_token();
    put_access_token(redis_conn, user, &token).await?;
    Ok(token)
}

// pub async fn check_access_token(
//     redis_conn: &deadpool_redis::Pool,
//     user: &str,
//     token: &str,
// ) -> bool {
//     let mut conn = redis_conn.get().await.unwrap();
//     let token_stored: String = match conn.get(access_token_redis(token)).await {
//         Ok(res) => res,

//     };
//     token_stored == Some(token.to_string())
// }
