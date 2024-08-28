//! define functions process the requests from clients directly

mod upload;

use super::{
    client_response::{get_status::GetStatusResponse, NewSessionResponse, UnregisterResponse},
    Connection, DBRequest,
};
use crate::{
    consts::ID,
    requests::{new_session::NewSession, upload::Upload},
};
use tokio::sync::{mpsc, oneshot};
use tungstenite::Message;

impl Connection {
    pub async fn unregister(
        id: ID,
        request_sender: &mpsc::Sender<DBRequest>,
        net_sender: &mpsc::Sender<Message>,
    ) -> anyhow::Result<()> {
        let channel = oneshot::channel();
        let unregister = DBRequest::Unregister {
            id,
            resp: channel.0,
        };
        request_sender.send(unregister).await?;
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
        request_sender: &mpsc::Sender<DBRequest>,
        net_sender: &mpsc::Sender<Message>,
        json: NewSession,
    ) -> anyhow::Result<()> {
        let channel = oneshot::channel();
        let new_session = DBRequest::NewSession {
            id,
            resp: channel.0,
        };
        request_sender.send(new_session).await?;
        let ret = channel.1.await?;
        let resp = match ret {
            Ok(resp) => resp,
            Err(status) => NewSessionResponse::failed(status),
        };
        net_sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await?;
        Ok(())
    }

    pub async fn get_status(net_sender: &mpsc::Sender<Message>) -> anyhow::Result<()> {
        let resp = GetStatusResponse::normal();
        let resp = serde_json::to_string(&resp)?;
        net_sender.send(Message::Text(resp)).await?;
        Ok(())
    }
}
