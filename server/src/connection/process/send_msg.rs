use crate::{
    DbPool,
    client::{MsgConvert, requests::UserSendMsgRequest, response::UserSendMsgResponse},
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
    match insert_msg_record(
        user_info.id,
        request.session_id,
        serde_json::value::to_value(request.bundle_msg).unwrap(),
        &db_pool.db_pool,
    )
    .await
    {
        Ok(msg_id) => {
            net_sender
                .send(UserSendMsgResponse::success(msg_id).to_msg())
                .await?
        }
        Err(e) => {
            tracing::error!("Database error:{e}");
            // net_sender
            //     .send(ErrorMsgResponse::server_error("Database error").to_msg())
            //     .await?
        }
    };
    Ok(())
}

async fn insert_msg_record(
    user_id: ID,
    session_id: ID,
    msg: serde_json::Value,
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
