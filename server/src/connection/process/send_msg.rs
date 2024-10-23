use crate::{
    DbPool,
    client::{
        MsgConvert,
        requests::{Status, UserSendMsgRequest},
        response::UserSendMsgResponse,
    },
    connection::{NetSender, UserInfo},
    consts::{ID, MsgID},
};
use derive::db_compatibility;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

pub async fn send_msg(
    user_info: &UserInfo,
    request: UserSendMsgRequest,
    net_sender: impl NetSender,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let msg = serde_json::to_string(&request.bundle_msg).unwrap();
    let ret = match insert_msg_record(user_info.id, request.session_id, msg, &db_pool.db_pool).await
    {
        Ok(msg_id) => UserSendMsgResponse::success(msg_id),
        Err(e) => {
            tracing::error!("Database error:{e}");
            UserSendMsgResponse::failure(Status::ServerError)
        }
    };
    net_sender.send(ret.to_msg()).await?;
    Ok(())
}

#[db_compatibility]
async fn insert_msg_record(
    user_id: ID,
    session_id: ID,
    msg: String,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<MsgID> {
    use entities::prelude::*;
    use entities::user_chat_msg;
    let msg = user_chat_msg::ActiveModel {
        ..Default::default()
    };
    let msg = msg.insert(db_conn).await?;
    Ok(msg.chat_msg_id.into())
}
