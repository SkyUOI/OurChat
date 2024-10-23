use crate::{
    DbPool,
    client::{MsgConvert, requests::GetUserMsgRequest, response::ErrorMsgResponse},
    connection::{NetSender, UserInfo},
    consts::ID,
};
use derive::db_compatibility;
use sea_orm::DatabaseConnection;

pub async fn get_user_msg(
    user_info: &UserInfo,
    request: GetUserMsgRequest,
    net_sender: impl NetSender,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let ret = match get_session_msgs(user_info.id, &db_pool.db_pool).await {
        Ok(_) => {
            todo!()
        }
        Err(e) => {
            tracing::error!("Database error:{e}");
            ErrorMsgResponse::new("Database error".to_owned()).to_msg()
        }
    };
    net_sender.send(ret).await?;
    Ok(())
}

#[db_compatibility]
async fn get_session_msgs(user_id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<()> {
    use entities::prelude::*;
    use entities::user_chat_msg;
    Ok(())
}
