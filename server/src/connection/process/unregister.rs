use crate::{
    client::{
        MsgConvert,
        response::{ErrorMsgResponse, UnregisterResponse},
    },
    consts::ID,
    entities::{prelude::*, session_relation, user, user_chat_msg},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, thiserror::Error)]
enum ErrorOfUnregister {
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),
}

/// Remove user from database
async fn remove_account(
    id: ID,
    db_connection: &DatabaseConnection,
) -> Result<(), ErrorOfUnregister> {
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    user.delete(db_connection).await?;
    Ok(())
}

/// Remove all session related to the user
async fn remove_session_record(
    id: ID,
    db_conn: &DatabaseConnection,
) -> Result<(), ErrorOfUnregister> {
    let id: u64 = id.into();
    SessionRelation::delete_many()
        .filter(session_relation::Column::UserId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

async fn remove_msgs_of_user(
    id: ID,
    db_conn: &DatabaseConnection,
) -> Result<(), ErrorOfUnregister> {
    let id: u64 = id.into();
    UserChatMsg::delete_many()
        .filter(user_chat_msg::Column::SenderId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

pub async fn unregister(
    id: ID,
    net_sender: &mpsc::Sender<Message>,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<()> {
    let batch = async {
        remove_session_record(id, db_conn).await?;
        remove_msgs_of_user(id, db_conn).await?;
        remove_account(id, db_conn).await?;
        Ok(())
    };
    let resp = match batch.await {
        Ok(_) => UnregisterResponse::new().to_msg(),
        Err(ErrorOfUnregister::DbError(e)) => {
            tracing::error!("Database error:{e}");
            ErrorMsgResponse::server_error("Database error").to_msg()
        }
        Err(ErrorOfUnregister::UnknownError(e)) => {
            tracing::error!("Unknown error:{e}");
            ErrorMsgResponse::server_error("Unknown error").to_msg()
        }
    };
    net_sender.send(resp).await?;
    net_sender.send(Message::Close(None)).await?;
    Ok(())
}
