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
pub use register::register;
pub use send_msg::send_msg;
pub use set_account_info::set_account_info;
pub use set_friend_info::set_friend_info;
pub use unregister::unregister;
pub use upload::upload;
