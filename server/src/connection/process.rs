//! define functions process the requests from clients directly

mod upload;
pub mod verify;

use super::{
    Connection,
    client_response::{NewSessionResponse, UnregisterResponse},
};
use crate::{component::EmailSender, consts::ID, requests::new_session::NewSession, server};
use sea_orm::DatabaseConnection;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

impl<T: EmailSender> Connection<T> {
    pub async fn unregister(
        id: ID,
        net_sender: &mpsc::Sender<Message>,
        db_conn: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        let ret = server::process::unregister(id, db_conn).await?;
        let resp = UnregisterResponse::new(ret);
        net_sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await?;
        net_sender.send(Message::Close(None)).await?;
        Ok(())
    }

    pub async fn new_session(
        id: ID,
        net_sender: &mpsc::Sender<Message>,
        _json: NewSession,
        db_conn: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        let resp = server::process::new_session(id, db_conn)
            .await?
            .unwrap_or_else(NewSessionResponse::failed);
        net_sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await?;
        Ok(())
    }
}
