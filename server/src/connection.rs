//! Process the connection to server

mod basic;
mod process;

use std::sync::Arc;

use crate::{
    DbPool, HttpSender, ShutdownRev, ShutdownSdr,
    client::{
        MsgConvert,
        requests::{
            self, AcceptSessionRequest, GetAccountInfoRequest, GetUserMsgRequest,
            SetAccountRequest, SetFriendInfoRequest, UserSendMsgRequest,
            new_session::NewSessionRequest, upload::UploadRequest,
        },
        response::{self, ErrorMsgResponse},
    },
    component::EmailSender,
    consts::{ID, MessageType},
    entities::{operations, prelude::*},
    server::httpserver::{VerifyRecord, verify::verify_client},
    shared_state,
};
use anyhow::{anyhow, bail};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use process::get_account_info;
use response::{get_status::GetStatusResponse, verify::VerifyResponse};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    net::TcpStream,
    select,
    sync::{Notify, mpsc},
};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;

pub mod db {
    pub use super::basic::get_id;
    pub use super::process::new_session::{add_to_session, batch_add_to_session, create_session};
}

type InComing = SplitStream<WS>;
type OutGoing = SplitSink<WS, Message>;

/// server-side websocket
pub type WS = WebSocketStream<TcpStream>;

struct UserInfo {
    id: ID,
    ocid: String,
}

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
    shutdown_sender: ShutdownSdr,
    http_sender: Option<HttpSender>,
    shared_data: Arc<crate::SharedData<T>>,
    dbpool: Option<DbPool>,
}
enum VerifyStatus {
    Success(UserInfo),
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

async fn get_requests(id: ID, db_conn: &DatabaseConnection) -> anyhow::Result<Vec<String>> {
    let id: u64 = id.into();
    let stored_requests = Operations::find()
        .filter(operations::Column::Id.eq(id))
        .all(db_conn)
        .await?;
    let mut ret = Vec::new();
    for i in stored_requests {
        if i.once {
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

async fn from_value<T: DeserializeOwned>(
    json: Value,
    net_sender: &impl NetSender,
) -> anyhow::Result<Option<T>> {
    match serde_json::from_value(json) {
        Ok(t) => Ok(Some(t)),
        Err(e) => {
            net_sender
                .send(
                    ErrorMsgResponse::new(
                        requests::Status::ArgOrInstNotCorrectError,
                        "wrong json structure".to_owned(),
                    )
                    .to_msg(),
                )
                .await?;
            tracing::trace!("wrong json structure: {}", e);
            Ok(None)
        }
    }
}

impl<T: EmailSender> Connection<T> {
    pub fn new(
        socket: WS,
        http_sender: HttpSender,
        shutdown_sender: ShutdownSdr,
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
    /// # Return
    /// if the action is not be executed, it will turn the json back to optimize the performance,otherwise it will consume the json and return None
    async fn low_level_action(
        user_info: Option<&UserInfo>,
        code: MessageType,
        json: Value,
        dbpool: &DbPool,
        net_sender: impl NetSender,
    ) -> anyhow::Result<Option<Value>> {
        match code {
            MessageType::GetStatus => {
                net_sender
                    .send(GetStatusResponse::normal().to_msg())
                    .await?;
            }
            MessageType::GetAccountInfo => {
                let json: GetAccountInfoRequest = match from_value(json, &net_sender).await? {
                    Some(json) => json,
                    None => return Ok(None),
                };
                get_account_info(user_info, net_sender, json, dbpool).await?;
            }
            _ => {
                return Ok(Some(json));
            }
        }
        Ok(None)
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
                .send(VerifyResponse::email_cannot_be_sent().to_msg())
                .await?;
        }
        let json: requests::VerifyRequest = from_value(json.clone(), &net_sender)
            .await?
            .ok_or(anyhow!(""))?;
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
            Ok(_) => net_sender.send(VerifyResponse::success().to_msg()).await?,
            Err(e) => {
                tracing::error!("Failed to verify email: {}", e);
                net_sender
                    .send(VerifyResponse::email_cannot_be_sent().to_msg())
                    .await?
            }
        };
        Ok(verify_success)
    }

    /// Setup when verifying,only allow low level operations to be executed
    async fn verify_notifying(
        user_info: Option<&UserInfo>,
        net_receiver: &mut InComing,
        net_sender: impl NetSender + Clone,
        dbpool: &DbPool,
    ) {
        loop {
            let msg = match net_receiver.next().await {
                Some(Ok(msg)) => msg,
                _ => break,
            };
            let (code, json) = match msg {
                Message::Text(text) => match get_code(&text) {
                    Ok(Some((code, json))) => (code, json),
                    _ => continue,
                },
                _ => {
                    continue;
                }
            };
            match Self::low_level_action(user_info, code, json, dbpool, net_sender.clone()).await {
                Ok(None) => {}
                Err(e) => {
                    tracing::error!("Failed to execute low level action: {}", e);
                }
                Ok(Some(_json)) => {
                    if let Err(e) = net_sender
                        .send(
                            response::ErrorMsgResponse::new(
                                requests::Status::RequestReject,
                                "Email has not been confirmed now",
                            )
                            .to_msg(),
                        )
                        .await
                    {
                        tracing::error!("Failed to send error msg: {}", e);
                    }
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
                        basic::send_error_msg(
                            net_send_closure,
                            requests::Status::ArgOrInstNotCorrectError,
                            "Not login or register code",
                        )
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
                        let json = match Self::low_level_action(
                            None,
                            code,
                            json,
                            dbpool,
                            net_send_closure,
                        )
                        .await?
                        {
                            Some(json) => json,
                            None => {
                                continue;
                            }
                        };
                        match code {
                            MessageType::Login => {
                                let login_data: requests::LoginRequest =
                                    match from_value(json, &net_send_closure).await? {
                                        None => {
                                            continue;
                                        }
                                        Some(data) => data,
                                    };
                                process::login::login_request(
                                    net_send_closure,
                                    login_data,
                                    &dbpool.db_pool,
                                )
                                .await?
                            }
                            MessageType::Register => {
                                let request_data: requests::RegisterRequest =
                                    match from_value(json, &net_send_closure).await? {
                                        None => {
                                            continue;
                                        }
                                        Some(data) => data,
                                    };
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
                                            _ = Self::verify_notifying(None, incoming, net_send_closure, dbpool) => {}
                                            _ = notifier.notified() => {}
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to verify email: {}", e);
                                    }
                                }
                                // send a email to client to show the verify status
                                let resp = VerifyResponse::success();
                                outgoing.send(resp.to_msg()).await?;
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

    async fn read_loop(
        mut incoming: InComing,
        user_info: UserInfo,
        net_sender: mpsc::Sender<Message>,
        http_sender: HttpSender,
        mut shutdown_receiver: ShutdownRev,
        shared_data: Arc<crate::SharedData<T>>,
        db_pool: DbPool,
    ) -> anyhow::Result<()> {
        let net_sender_closure = |data| async {
            net_sender.send(data).await?;
            Ok(())
        };

        let work = async {
            'con_loop: loop {
                let msg = match incoming.next().await {
                    None => break,
                    Some(res) => match res {
                        Ok(msg) => {
                            tracing::trace!("recv msg:{}", msg);
                            msg
                        }
                        Err(e) => Err(e)?,
                    },
                };

                match msg {
                    Message::Text(msg) => {
                        let (code, json) = match get_code(&msg) {
                            Ok(Some((code, json))) => (code, json),
                            _ => {
                                basic::send_error_msg(
                                    net_sender_closure,
                                    requests::Status::ArgOrInstNotCorrectError,
                                    "code is not correct",
                                )
                                .await?;
                                continue 'con_loop;
                            }
                        };
                        let json = match Self::low_level_action(
                            Some(&user_info),
                            code,
                            json,
                            &db_pool,
                            net_sender_closure,
                        )
                        .await?
                        {
                            Some(json) => json,
                            None => continue,
                        };
                        match code {
                            MessageType::Unregister => {
                                process::unregister(user_info.id, &net_sender, &db_pool.db_pool)
                                    .await?;
                                continue 'con_loop;
                            }
                            MessageType::NewSession => {
                                let json: NewSessionRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(data) => data,
                                    };
                                process::new_session(
                                    &user_info,
                                    net_sender_closure,
                                    json,
                                    &db_pool,
                                    &shared_data,
                                )
                                .await?;
                                continue 'con_loop;
                            }
                            MessageType::Upload => {
                                let mut json: UploadRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(json) => json,
                                    };
                                // Generate url first and then reply
                                let hash = json.hash.clone();
                                let auto_clean = json.auto_clean;
                                let (send, key) = process::upload(
                                    user_info.id,
                                    &net_sender,
                                    &mut json,
                                    &db_pool.db_pool,
                                )
                                .await?;
                                send.await?;
                                continue 'con_loop;
                            }
                            MessageType::AcceptSession => {
                                let json: AcceptSessionRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(data) => data,
                                    };
                                process::accept_session(
                                    user_info.id,
                                    net_sender_closure,
                                    json,
                                    &db_pool,
                                )
                                .await?;
                                continue 'con_loop;
                            }
                            MessageType::SetAccountInfo => {
                                let json: SetAccountRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(data) => data,
                                    };
                                process::set_account_info(
                                    &user_info,
                                    net_sender_closure,
                                    json,
                                    &db_pool,
                                )
                                .await?;
                                continue 'con_loop;
                            }
                            MessageType::SetFriendInfo => {
                                let json: SetFriendInfoRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(data) => data,
                                    };
                                process::set_friend_info(
                                    &user_info,
                                    json,
                                    net_sender_closure,
                                    &db_pool,
                                )
                                .await?;
                                continue 'con_loop;
                            }
                            MessageType::UserSendMsg => {
                                let json: UserSendMsgRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(data) => data,
                                    };
                                process::send_msg(&user_info, json, net_sender_closure, &db_pool)
                                    .await?;
                                continue 'con_loop;
                            }
                            MessageType::GetUserMsg => {
                                let json: GetUserMsgRequest =
                                    match from_value(json, &net_sender_closure).await? {
                                        None => {
                                            continue 'con_loop;
                                        }
                                        Some(data) => data,
                                    };
                                process::get_user_msg(
                                    &user_info,
                                    json,
                                    net_sender_closure,
                                    &db_pool,
                                )
                                .await?;
                                continue 'con_loop;
                            }
                            _ => {
                                basic::send_error_msg(
                                    net_sender_closure,
                                    requests::Status::ArgOrInstNotCorrectError,
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
            _ = shutdown_receiver.wait_shutdowning() => {
                Ok(())
            }
        }
    }

    pub async fn write_loop(
        mut outgoing: OutGoing,
        mut receiver: mpsc::Receiver<Message>,
        mut shutdown_receiver: ShutdownRev,
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
            _ = shutdown_receiver.wait_shutdowning() => {
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
                                    socket.send(resp.to_msg()).await?;
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
        let mut shutdown_receiver = self.shutdown_sender.new_receiver("connection", "");
        let user_info;
        let http_sender = self.http_sender.take().unwrap();
        let dbpool = self.dbpool.take().unwrap();
        // start maintaining loop
        if shared_state::get_maintaining() {
            tracing::info!("start maintaining loop");
            Self::maintaining(&mut socket).await?;
            return Ok(());
        }
        let (outgoing, mut incoming) = socket.split();
        let (msg_sender, msg_receiver) = mpsc::channel(32);
        tokio::spawn(Self::write_loop(
            outgoing,
            msg_receiver,
            self.shutdown_sender
                .new_receiver("connection write loop", ""),
        ));
        tracing::debug!("start verify");
        select! {
            ret = Connection::verify(
                &mut incoming,
                msg_sender.clone(),
                self.shared_data.clone(),
                &dbpool
            ) => {
                user_info = match ret? {
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
            _ = shutdown_receiver.wait_shutdowning() => {
                return Ok(());
            }
        }
        let requests = get_requests(user_info.id, &dbpool.db_pool).await?;
        for i in requests {
            msg_sender.send(Message::Text(i)).await?;
        }
        let _guard =
            ConnectionGuard::new(user_info.id, msg_sender.clone(), self.shared_data.clone());
        Self::read_loop(
            incoming,
            user_info,
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

/// Used to do some cleanup
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
