//! Process the connection to server

mod basic;
mod process;

use std::sync::Arc;

use crate::{
    DbPool, HttpSender,
    client::{
        requests::{self, AcceptSession, new_session::NewSession, upload::Upload},
        response,
    },
    component::EmailSender,
    consts::{ID, MessageType},
    db::BooleanLike,
    entities::mysql::operations,
    server::httpserver::{FileRecord, VerifyRecord, verify::verify_client},
    shared_state,
};
use anyhow::bail;
use clap::Id;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use process::new_session::mapped_to_operations;
use redis::AsyncCommands;
use response::{get_status::GetStatusResponse, verify::VerifyResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    select,
    sync::{Notify, broadcast, mpsc},
};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;

type InComing = SplitStream<WS>;
type OutGoing = SplitSink<WS, Message>;

/// server-side websocket
pub type WS = WebSocketStream<TcpStream>;

pub trait NetSender {
    type Fut: Future<Output = anyhow::Result<()>>;
    fn send(&self, msg: Message) -> Self::Fut;
}

impl<F, Fut> NetSender for F
where
    F: Fn(Message) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    type Fut = Fut;
    fn send(&self, msg: Message) -> Self::Fut {
        (self)(msg)
    }
}

/// Connection to a client
pub struct Connection<T: EmailSender + 'static> {
    socket: Option<WebSocketStream<TcpStream>>,
    shutdown_sender: broadcast::Sender<()>,
    http_sender: Option<HttpSender>,
    shared_data: Arc<crate::SharedData<T>>,
    dbpool: Option<DbPool>,
}
enum VerifyStatus {
    Success(ID),
    Fail,
}

fn get_code(s: &str) -> anyhow::Result<Option<(MessageType, Value)>> {
    let json: Value = serde_json::from_str(s)?;
    let code = &json["code"];
    if let Value::Number(code) = code {
        let code = code.as_u64();
        if let Some(code) = code {
            if let Ok(msg) = MessageType::try_from(code as i32) {
                return Ok(Some((msg, json)));
            }
        }
    }
    Ok(None)
}

#[derive::db_compatibility]
async fn get_requests(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<Vec<String>> {
    use entities::operations;
    use entities::prelude::*;
    let id: u64 = id.into();
    let stored_requests = Operations::find()
        .filter(operations::Column::Id.eq(id))
        .all(db_conn)
        .await?;
    let mut ret = Vec::new();
    for i in stored_requests {
        if i.once.is_true() {
            Operations::delete_by_id(i.oper_id).exec(db_conn).await?;
        }
        if i.expires_at < chrono::Utc::now() {
            Operations::delete_by_id(i.oper_id).exec(db_conn).await?;
            continue;
        }
        ret.push(i.operation);
    }
    Ok(ret)
}

impl<T: EmailSender> Connection<T> {
    pub fn new(
        socket: WS,
        http_sender: HttpSender,
        shutdown_sender: broadcast::Sender<()>,
        shared_data: Arc<crate::SharedData<T>>,
        dbpool: DbPool,
    ) -> Self {
        Self {
            socket: Some(socket),
            http_sender: Some(http_sender),
            shutdown_sender,
            shared_data,
            dbpool: Some(dbpool),
        }
    }

    /// Operate low level actions which are allowed all the time
    async fn low_level_action(
        code: MessageType,
        net_sender: impl NetSender,
    ) -> anyhow::Result<bool> {
        match code {
            MessageType::GetStatus => {
                net_sender.send(GetStatusResponse::normal().into()).await?;
                return Ok(true);
            }
            _ => (),
        }
        Ok(false)
    }

    /// Send a Email to verify the user
    async fn email_verify(
        json: &Value,
        shared_data: &Arc<crate::SharedData<impl EmailSender>>,
        dbpool: &DbPool,
        net_sender: impl NetSender,
    ) -> anyhow::Result<Arc<Notify>> {
        if shared_data.email_client.is_none() {
            net_sender
                .send(VerifyResponse::email_cannot_be_sent().into())
                .await?;
        }
        let json: requests::Verify = serde_json::from_value(json.clone())?;
        let verify_success = Arc::new(Notify::new());
        // this message's meaning is the email has been sent
        match verify_client(
            dbpool,
            shared_data.clone(),
            VerifyRecord::new(json.email, process::verify::generate_token()),
            verify_success.clone(),
        )
        .await
        {
            Ok(_) => net_sender.send(VerifyResponse::success().into()).await?,
            Err(e) => {
                tracing::error!("Failed to verify email: {}", e);
                net_sender
                    .send(VerifyResponse::email_cannot_be_sent().into())
                    .await?
            }
        };
        Ok(verify_success)
    }

    /// Setup when verifying,only allow low level operations to be executed
    async fn verify_notifying(net_receiver: &mut InComing, net_sender: impl NetSender + Clone) {
        loop {
            let msg = match net_receiver.next().await {
                Some(Ok(msg)) => msg,
                _ => break,
            };
            let (code, _json) = match msg {
                Message::Text(text) => match get_code(&text) {
                    Ok(Some((code, json))) => (code, json),
                    _ => continue,
                },
                _ => {
                    continue;
                }
            };
            match Self::low_level_action(code, net_sender.clone()).await {
                Ok(executed) => {
                    if !executed {
                        if let Err(e) = net_sender
                            .send(
                                response::ErrorMsgResponse::new(
                                    "Email has not been confirmed now".to_owned(),
                                )
                                .into(),
                            )
                            .await
                        {
                            tracing::error!("Failed to send error msg: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute low level action: {}", e);
                }
            }
        }
    }

    /// Verify Client
    /// # Note
    /// We allow some low level actions to be executed
    async fn verify(
        incoming: &mut InComing,
        outgoing: mpsc::Sender<Message>,
        shared_data: Arc<crate::SharedData<T>>,
        dbpool: &DbPool,
    ) -> anyhow::Result<Option<VerifyStatus>> {
        let net_send_closure = |t| async {
            outgoing.send(t).await?;
            Ok(())
        };
        // Try to fetch previous verification
        loop {
            let msg = match incoming.next().await {
                None => {
                    bail!("Failed to receive message when logining");
                }
                Some(res) => match res {
                    Ok(msg) => msg,
                    Err(e) => Err(e)?,
                },
            };
            match msg {
                Message::Text(text) => {
                    let verify_failed = || async {
                        basic::send_error_msg(net_send_closure, "Not login or register code")
                            .await?;
                        tracing::info!(
                            "Failed to login,msg is {:?},not login or register code",
                            text
                        );
                        anyhow::Ok(VerifyStatus::Fail)
                    };
                    // Get message type
                    let (code, json) = match get_code(&text) {
                        Ok(Some((code, json))) => (code, json),
                        _ => {
                            verify_failed().await?;
                            continue;
                        }
                    };

                    let status = {
                        if Self::low_level_action(code, |t| async {
                            outgoing.send(t).await?;
                            Ok(())
                        })
                        .await?
                        {
                            continue;
                        }
                        match code {
                            MessageType::Login => {
                                let login_data: requests::Login = serde_json::from_value(json)?;
                                process::login::login_request(
                                    net_send_closure,
                                    login_data,
                                    &dbpool.db_pool,
                                )
                                .await?
                            }
                            MessageType::Register => {
                                let request_data: requests::Register =
                                    serde_json::from_value(json)?;
                                process::register(net_send_closure, request_data, &dbpool.db_pool)
                                    .await?
                            }
                            MessageType::Verify => {
                                tracing::info!("Start to verify email");
                                match Self::email_verify(&json, &shared_data, dbpool, |msg| async {
                                    outgoing.send(msg).await?;
                                    anyhow::Ok(())
                                })
                                .await
                                {
                                    Ok(notifier) => {
                                        select! {
                                            _ = Self::verify_notifying(incoming, |msg| async {
                                                outgoing.send(msg).await?;
                                                anyhow::Ok(())
                                            }) => {}
                                            _ = notifier.notified() => {}
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to verify email: {}", e);
                                    }
                                }
                                // send a email to client to show the verify status
                                let resp = VerifyResponse::success();
                                outgoing.send(resp.into()).await?;
                                tracing::info!("End to verify email");
                                continue;
                            }
                            _ => {
                                verify_failed().await?;
                                continue;
                            }
                        }
                    };
                    match status {
                        VerifyStatus::Success(_) => {
                            return Ok(Some(status));
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                Message::Close(_) => {
                    return Ok(None);
                }
                _ => {
                    bail!("Failed to login,not a text message");
                }
            }
        }
    }

    pub async fn read_loop(
        mut incoming: InComing,
        id: ID,
        net_sender: mpsc::Sender<Message>,
        http_sender: HttpSender,
        mut shutdown_receiver: broadcast::Receiver<()>,
        shared_data: Arc<crate::SharedData<T>>,
        dbpool: DbPool,
    ) -> anyhow::Result<()> {
        let net_send_closure = |data| async {
            net_sender.send(data).await?;
            Ok(())
        };
        let mut requests = get_requests(id, &dbpool.db_pool).await?.into_iter();

        let work = async {
            'con_loop: loop {
                let msg = if let Some(request) = requests.next() {
                    Message::Text(request)
                } else {
                    let msg = incoming.next().await;
                    if msg.is_none() {
                        break;
                    }
                    let msg = msg.unwrap();

                    match msg {
                        Ok(msg) => {
                            tracing::debug!("recv msg:{}", msg);
                            msg
                        }
                        Err(e) => Err(e)?,
                    }
                };

                match msg {
                    Message::Text(msg) => {
                        let (code, json) = match get_code(&msg) {
                            Ok(Some((code, json))) => (code, json),
                            _ => {
                                basic::send_error_msg(net_send_closure, "codei is not correct")
                                    .await?;
                                continue 'con_loop;
                            }
                        };
                        if Self::low_level_action(code, |t| async {
                            net_sender.send(t).await?;
                            Ok(())
                        })
                        .await?
                        {
                            continue;
                        }
                        match code {
                            MessageType::Unregister => {
                                process::unregister(id, &net_sender, &dbpool.db_pool).await?;
                                continue 'con_loop;
                            }
                            MessageType::NewSession => {
                                let json: NewSession = match serde_json::from_value(json) {
                                    Err(_) => {
                                        tracing::warn!("Wrong json structure");
                                        continue 'con_loop;
                                    }
                                    Ok(data) => data,
                                };
                                process::new_session(
                                    id,
                                    net_send_closure,
                                    json,
                                    &dbpool,
                                    &shared_data,
                                )
                                .await?;
                                continue 'con_loop;
                            }
                            MessageType::Upload => {
                                let mut json: Upload = match serde_json::from_value(json) {
                                    Err(_) => {
                                        tracing::warn!("Wrong json structure");
                                        continue 'con_loop;
                                    }
                                    Ok(json) => json,
                                };
                                // Generate url first and then reply
                                let hash = json.hash.clone();
                                let auto_clean = json.auto_clean;
                                let (send, key) =
                                    process::upload(id, &net_sender, &mut json, &dbpool.db_pool)
                                        .await?;
                                let record = FileRecord::new(key, hash, auto_clean, id);
                                http_sender.file_record.send(record).await?;
                                send.await?;
                                continue 'con_loop;
                            }
                            MessageType::AcceptSession => {
                                let json: AcceptSession = match serde_json::from_value(json) {
                                    Err(_) => {
                                        tracing::warn!("Wrong json structure");
                                        continue 'con_loop;
                                    }
                                    Ok(data) => data,
                                };
                                process::accept_session(id, net_send_closure, json, &dbpool)
                                    .await?;
                                continue 'con_loop;
                            }
                            _ => {
                                basic::send_error_msg(
                                    net_send_closure,
                                    format!("Not a supported code {:?}", code),
                                )
                                .await?;
                            }
                        }
                    }
                    Message::Ping(_) => {
                        net_sender.send(Message::Pong(vec![])).await?;
                    }
                    Message::Pong(_) => {
                        tracing::info!("recv pong");
                    }
                    Message::Close(_) => {
                        break 'con_loop;
                    }
                    _ => {}
                }
            }
            tracing::debug!("connection closed");
            Ok(())
        };
        select! {
            ret = work => {ret},
            _ = shutdown_receiver.recv() => {
                Ok(())
            }
        }
    }

    pub async fn write_loop(
        mut outgoing: OutGoing,
        mut receiver: mpsc::Receiver<Message>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        let work = async {
            while let Some(msg) = receiver.recv().await {
                tracing::debug!("send msg:{}", msg);
                outgoing.send(msg).await?;
                tracing::debug!("send successful");
            }
            outgoing.close().await?;
            Ok(())
        };
        select! {
            ret = work => {ret},
            _ = shutdown_receiver.recv() => {
                Ok(())
            }
        }
    }

    async fn maintaining(socket: &mut WS) -> anyhow::Result<()> {
        while let Some(msg) = socket.next().await {
            match msg {
                Ok(msg) => match msg {
                    Message::Text(msg) => {
                        let json: Value = serde_json::from_str(&msg)?;
                        let code = &json["code"];
                        if let Value::Number(code) = code {
                            let code = code.as_u64();
                            if let Some(code) = code {
                                if code == MessageType::GetStatus as u64 {
                                    let resp = GetStatusResponse::maintaining();
                                    socket.send(resp.into()).await?;
                                    tracing::debug!("Send maintaining msg");
                                }
                            } else {
                                bail!("Wrong json msg code");
                            }
                        } else {
                            bail!("Wrong json structure");
                        }
                    }
                    Message::Ping(_) => {
                        socket.send(Message::Pong(vec![])).await?;
                    }
                    Message::Pong(_) => {
                        tracing::info!("recv pong");
                    }
                    _ => {}
                },
                Err(e) => Err(e)?,
            };
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn work(&mut self) -> anyhow::Result<()> {
        let mut socket = self.socket.take().unwrap();
        let mut shutdown_receiver = self.shutdown_sender.subscribe();
        let id;
        let http_sender = self.http_sender.take().unwrap();
        let dbpool = self.dbpool.take().unwrap();
        // start maintaining loop
        if shared_state::get_maintaining() {
            Self::maintaining(&mut socket).await?;
            return Ok(());
        }
        let (outgoing, mut incoming) = socket.split();
        let (msg_sender, msg_receiver) = mpsc::channel(32);
        tokio::spawn(Self::write_loop(
            outgoing,
            msg_receiver,
            self.shutdown_sender.subscribe(),
        ));
        select! {
            ret = Connection::verify(
                &mut incoming,
                msg_sender.clone(),
                self.shared_data.clone(),
                &dbpool
            ) => {
                id = match ret? {
                    Some(ret) => {
                        match ret {
                            VerifyStatus::Success(id) => id,
                            VerifyStatus::Fail => {
                                return Ok(())
                            }
                        }
                    }
                    None => return Ok(())
                };
            }
            _ = shutdown_receiver.recv() => {
                    return Ok(());
            }
        }
        let _guard = ConnectionGuard::new(id, msg_sender.clone(), self.shared_data.clone());
        Self::read_loop(
            incoming,
            id,
            msg_sender,
            http_sender,
            shutdown_receiver,
            self.shared_data.clone(),
            dbpool,
        )
        .await?;
        Ok(())
    }
}

struct ConnectionGuard<T: EmailSender> {
    id: ID,
    shared_data: Arc<crate::SharedData<T>>,
}

impl<T: EmailSender> ConnectionGuard<T> {
    pub fn new(
        id: ID,
        msg_sender: mpsc::Sender<Message>,
        shared_data: Arc<crate::SharedData<T>>,
    ) -> Self {
        shared_data.connected_clients.insert(id, msg_sender.clone());
        Self { id, shared_data }
    }
}

impl<T: EmailSender> Drop for ConnectionGuard<T> {
    fn drop(&mut self) {
        self.shared_data.connected_clients.remove(&self.id);
    }
}
