use crate::{
    client::{MsgConvert, requests, response::UnregisterResponse},
    consts::ID,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[derive::db_compatibility]
async fn remove_account(
    id: ID,
    db_connection: &DatabaseConnection,
) -> anyhow::Result<requests::Status> {
    use entities::user::ActiveModel as UserModel;
    let user = UserModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    match user.delete(db_connection).await {
        Ok(_) => Ok(requests::Status::Success),
        Err(e) => {
            tracing::error!("Database error:{e}");
            Ok(requests::Status::ServerError)
        }
    }
}
pub async fn unregister(
    id: ID,
    net_sender: &mpsc::Sender<Message>,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<()> {
    let ret = remove_account(id, db_conn).await?;
    let resp = UnregisterResponse::new(ret);
    net_sender.send(resp.to_msg()).await?;
    net_sender.send(Message::Close(None)).await?;
    Ok(())
}
