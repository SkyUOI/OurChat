//! define functions process the requests from clients directly

mod upload;
pub mod verify;

use super::{
    Connection, DBRequest,
    client_response::{NewSessionResponse, UnregisterResponse},
};
use crate::{consts::ID, requests::new_session::NewSession};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::protocol::Message;

impl Connection {
    pub async fn unregister(
        id: ID,
        db_sender: &mpsc::Sender<DBRequest>,
        net_sender: &mpsc::Sender<Message>,
    ) -> anyhow::Result<()> {
        let channel = oneshot::channel();
        let unregister = DBRequest::Unregister {
            id,
            resp: channel.0,
        };
        db_sender.send(unregister).await?;
        let ret = channel.1.await?;
        let resp = UnregisterResponse::new(ret);
        net_sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await?;
        net_sender.send(Message::Close(None)).await?;
        Ok(())
    }

    pub async fn new_session(
        id: ID,
        db_sender: &mpsc::Sender<DBRequest>,
        net_sender: &mpsc::Sender<Message>,
        json: NewSession,
    ) -> anyhow::Result<()> {
        let channel = oneshot::channel();
        let new_session = DBRequest::NewSession {
            id,
            resp: channel.0,
        };
        db_sender.send(new_session).await?;
        let ret = channel.1.await?;
        let resp = ret.unwrap_or_else(NewSessionResponse::failed);
        net_sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await?;
        Ok(())
    }
}
