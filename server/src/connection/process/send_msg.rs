use crate::{
    DbPool,
    client::{
        MsgConvert,
        requests::UserSendMsgRequest,
        response::{ErrorMsgResponse, UserSendMsgResponse},
    },
    connection::{NetSender, UserInfo},
    consts::{ID, MsgID},
    entities::user_chat_msg,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};

pub async fn send_msg(
    user_info: &UserInfo,
    request: UserSendMsgRequest,
    net_sender: impl NetSender,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let msg = serde_json::to_string(&request.bundle_msg).unwrap();
    match insert_msg_record(user_info.id, request.session_id, msg, &db_pool.db_pool).await {
        Ok(msg_id) => {
            net_sender
                .send(UserSendMsgResponse::success(msg_id).to_msg())
                .await?
        }
        Err(e) => {
            tracing::error!("Database error:{e}");
            net_sender
                .send(ErrorMsgResponse::server_error("Database error").to_msg())
                .await?
        }
    };
    Ok(())
}

async fn insert_msg_record(
    user_id: ID,
    session_id: ID,
    msg: String,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<MsgID> {
    let msg = user_chat_msg::ActiveModel {
        msg_data: ActiveValue::Set(msg),
        sender_id: ActiveValue::Set(user_id.into()),
        session_id: ActiveValue::Set(session_id.into()),
        ..Default::default()
    };
    let msg = msg.insert(db_conn).await?;
    Ok(msg.chat_msg_id.into())
}
