use crate::{
    client::{MsgConvert, requests, response::UnregisterResponse},
    consts::ID,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

/// Remove user from database
#[derive::db_compatibility]
async fn remove_account(id: ID, db_connection: &DatabaseConnection) -> anyhow::Result<()> {
    use entities::user::ActiveModel as UserModel;
    let user = UserModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    user.delete(db_connection).await?;
    Ok(())
}

/// Remove all session related to the user
#[derive::db_compatibility]
async fn remove_session_record(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<()> {
    use entities::session_relation;
    let id: u64 = id.into();
    session_relation::Entity::delete_many()
        .filter(session_relation::Column::UserId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

#[derive::db_compatibility]
async fn remove_msgs_of_user(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<()> {
    use entities::user_chat_msg;
    let id: u64 = id.into();
    user_chat_msg::Entity::delete_many()
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
        anyhow::Ok(())
    };
    let status = match batch.await {
        Ok(_) => requests::Status::Success,
        Err(e) => {
            tracing::error!("Database error:{e}");
            requests::Status::ServerError
        }
    };
    let resp = UnregisterResponse::new(status);
    net_sender.send(resp.to_msg()).await?;
    net_sender.send(Message::Close(None)).await?;
    Ok(())
}
