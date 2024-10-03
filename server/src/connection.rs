//! Process the connection to server

mod basic;
pub mod client_response;
mod process;

use std::sync::Arc;

use crate::{
    HttpSender,
    component::EmailSender,
    consts::{Bt, ID, MessageType},
    requests::{self, new_session::NewSession, upload::Upload},
    server::httpserver::{FileRecord, VerifyRecord},
    shared_state,
};
use anyhow::bail;
use client_response::{
    LoginResponse, NewSessionResponse, RegisterResponse, get_status::GetStatusResponse,
    verify::VerifyResponse,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    select,
    sync::{broadcast, mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;

type InComing = SplitStream<WS>;
type OutGoing = SplitSink<WS, Message>;

/// Request that will be sent to database process loop and get the process response
pub enum DBRequest {
    Login {
        request: requests::Login,
        resp: oneshot::Sender<Result<(LoginResponse, ID), requests::Status>>,
    },
    Register {
        request: requests::Register,
        resp: oneshot::Sender<Result<(RegisterResponse, ID), requests::Status>>,
    },
    Unregister {
        id: ID,
        resp: oneshot::Sender<requests::Status>,
    },
    NewSession {
        id: ID,
        resp: oneshot::Sender<Result<NewSessionResponse, requests::Status>>,
    },
    UpLoad {
        id: ID,
        sz: Bt,
        resp: oneshot::Sender<requests::Status>,
    },
}

/// server-side websocket
pub type WS = WebSocketStream<TcpStream>;

/// Connection to a client
pub struct Connection<T: EmailSender + 'static> {
    socket: Option<WebSocketStream<TcpStream>>,
    shutdown_sender: broadcast::Sender<()>,
    request_sender: mpsc::Sender<DBRequest>,
    http_sender: Option<HttpSender>,
    shared_data: Arc<crate::SharedData<T>>,
}
enum VerifyStatus {
    Success(ID),
    Fail,
}

impl<T: EmailSender> Connection<T> {
    pub fn new(
        socket: WS,
        http_sender: HttpSender,
        shutdown_sender: broadcast::Sender<()>,
        request_sender: mpsc::Sender<DBRequest>,
        shared_data: Arc<crate::SharedData<T>>,
    ) -> Self {
        Self {
            socket: Some(socket),
            http_sender: Some(http_sender),
            shutdown_sender,
            request_sender,
            shared_data,
        }
    }

    /// Login Request
    async fn login_request(
        request_sender: &mpsc::Sender<DBRequest>,
        login_data: requests::Login,
    ) -> anyhow::Result<(String, VerifyStatus)> {
        let channel = oneshot::channel();
        let request = DBRequest::Login {
            request: login_data,
            resp: channel.0,
        };
        request_sender.send(request).await?;
        match channel.1.await? {
            Ok(ok_resp) => Ok((
                serde_json::to_string(&ok_resp.0)?,
                VerifyStatus::Success(ok_resp.1),
            )),
            Err(e) => Ok((
                serde_json::to_string(&LoginResponse::failed(e))?,
                VerifyStatus::Fail,
            )),
        }
    }

    /// Register Request
    async fn register_request(
        request_sender: &mpsc::Sender<DBRequest>,
        register_data: requests::Register,
    ) -> anyhow::Result<(String, VerifyStatus)> {
        let channel = oneshot::channel();
        let request = DBRequest::Register {
            request: register_data,
            resp: channel.0,
        };
        request_sender.send(request).await?;
        match channel.1.await? {
            Ok(ok_resp) => Ok((
                serde_json::to_string(&ok_resp.0)?,
                VerifyStatus::Success(ok_resp.1),
            )),
            Err(e) => Ok((
                serde_json::to_string(&RegisterResponse::failed(e))?,
                VerifyStatus::Fail,
            )),
        }
    }

    /// Operate low level actions which are allowed all the time
    async fn low_level_action(
        code: MessageType,
        json: &Value,
        http_sender: &mut HttpSender,
        shared_data: &Arc<crate::SharedData<impl EmailSender>>,
    ) -> anyhow::Result<Option<String>> {
        match code {
            MessageType::GetStatus => {
                Ok(Some(serde_json::to_string(&GetStatusResponse::normal())?))
            }
            MessageType::Verify => {
                if shared_data.email_client.is_none() {
                    return Ok(Some(serde_json::to_string(
                        &VerifyResponse::email_cannot_be_sent(),
                    )?));
                }
                let json: requests::Verify = serde_json::from_value(json.clone())?;
                let (resp_sender, resp_receiver) = oneshot::channel();
                http_sender
                    .verify_record
                    .send((
                        VerifyRecord::new(json.email, process::verify::generate_token()),
                        resp_sender,
                    ))
                    .await?;
                // this message's meaning is the email has been sent
                match resp_receiver.await {
                    Ok(Ok(_)) => Ok(Some(serde_json::to_string(&VerifyResponse::success())?)),
                    Ok(Err(e)) => {
                        tracing::error!("Failed to verify email: {}", e);
                        Ok(Some(serde_json::to_string(
                            &VerifyResponse::email_cannot_be_sent(),
                        )?))
                    }
                    Err(e) => Err(e)?,
                }
            }
            _ => Ok(None),
        }
    }

    /// Verify Client
    /// # Note
    /// We allow some low level actions to be executed
    async fn verify(
        incoming: &mut InComing,
        outgoing: mpsc::Sender<Message>,
        request_sender: &mpsc::Sender<DBRequest>,
        http_sender: &mut HttpSender,
        shared_data: Arc<crate::SharedData<T>>,
    ) -> anyhow::Result<Option<VerifyStatus>> {
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
                    let json: Value = serde_json::from_str(&text)?;
                    // Get message type
                    let code = &json["code"];
                    if let Value::Number(code) = code {
                        let code = code.as_u64();
                        let verify_failed = || async {
                            Self::send_error_msg(
                                |msg| async {
                                    outgoing.send(msg).await?;
                                    anyhow::Ok(())
                                },
                                "Not login or register code",
                            )
                            .await?;
                            tracing::info!(
                                "Failed to login,code is {:?},not login or register code",
                                code
                            );
                            anyhow::Ok(VerifyStatus::Fail)
                        };
                        let (resp_content, status) = match code {
                            None => {
                                verify_failed().await?;
                                continue;
                            }
                            Some(code) => {
                                let code = match MessageType::try_from(code as i32) {
                                    Ok(c) => c,
                                    Err(_) => {
                                        verify_failed().await?;
                                        continue;
                                    }
                                };
                                if let Some(resp) =
                                    Self::low_level_action(code, &json, http_sender, &shared_data)
                                        .await?
                                {
                                    outgoing.send(Message::Text(resp)).await?;
                                    continue;
                                }
                                match code {
                                    MessageType::Login => {
                                        let login_data: requests::Login =
                                            serde_json::from_value(json)?;
                                        Self::login_request(request_sender, login_data).await?
                                    }
                                    MessageType::Register => {
                                        let request_data: requests::Register =
                                            serde_json::from_value(json)?;
                                        Self::register_request(request_sender, request_data).await?
                                    }
                                    _ => {
                                        verify_failed().await?;
                                        continue;
                                    }
                                }
                            }
                        };
                        outgoing.send(Message::Text(resp_content)).await?;
                        match status {
                            VerifyStatus::Success(_) => {
                                return Ok(Some(status));
                            }
                            _ => {
                                continue;
                            }
                        }
                    } else {
                        bail!("Failed to login,code is not a number or missing");
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
        db_sender: mpsc::Sender<DBRequest>,
        mut http_sender: HttpSender,
        mut shutdown_receiver: broadcast::Receiver<()>,
        shared_data: Arc<crate::SharedData<T>>,
    ) -> anyhow::Result<()> {
        let net_send_closure = |data| async {
            net_sender.send(data).await?;
            Ok(())
        };
        let work = async {
            'con_loop: loop {
                let msg = incoming.next().await;
                if msg.is_none() {
                    break;
                }
                let msg = msg.unwrap();
                let msg = match msg {
                    Ok(msg) => {
                        tracing::debug!("recv msg:{}", msg);
                        msg
                    }
                    Err(e) => Err(e)?,
                };
                match msg {
                    Message::Text(msg) => {
                        let json: Value = serde_json::from_str(&msg)?;
                        let code = &json["code"];
                        if let Value::Number(code) = code {
                            let code = code.as_u64();
                            match code {
                                None => {
                                    Self::send_error_msg(
                                        net_send_closure,
                                        "code is not a unsigned number",
                                    )
                                    .await?;
                                }
                                Some(num) => {
                                    let code = match MessageType::try_from(num as i32) {
                                        Ok(num) => num,
                                        Err(_) => {
                                            Self::send_error_msg(
                                                net_send_closure,
                                                format!("Not a valid code {}", num),
                                            )
                                            .await?;
                                            continue 'con_loop;
                                        }
                                    };
                                    if let Some(oper_success) = Self::low_level_action(
                                        code,
                                        &json,
                                        &mut http_sender,
                                        &shared_data,
                                    )
                                    .await?
                                    {
                                        net_sender.send(Message::Text(oper_success)).await?;
                                        continue;
                                    }
                                    match code {
                                        MessageType::Unregister => {
                                            Self::unregister(id, &db_sender, &net_sender).await?;
                                            continue 'con_loop;
                                        }
                                        MessageType::NewSession => {
                                            let json: NewSession =
                                                match serde_json::from_value(json) {
                                                    Err(_) => {
                                                        tracing::warn!("Wrong json structure");
                                                        continue 'con_loop;
                                                    }
                                                    Ok(data) => data,
                                                };
                                            Self::new_session(id, &db_sender, &net_sender, json)
                                                .await?;
                                            continue 'con_loop;
                                        }
                                        MessageType::Upload => {
                                            let mut json: Upload =
                                                match serde_json::from_value(json) {
                                                    Err(_) => {
                                                        tracing::warn!("Wrong json structure");
                                                        continue 'con_loop;
                                                    }
                                                    Ok(json) => json,
                                                };
                                            // 先生成url再回复
                                            let hash = json.hash.clone();
                                            let auto_clean = json.auto_clean;
                                            let (send, key) = Self::upload(
                                                id,
                                                &db_sender,
                                                &net_sender,
                                                &mut json,
                                            )
                                            .await?;
                                            let record = FileRecord::new(key, hash, auto_clean, id);
                                            http_sender.file_record.send(record).await?;
                                            send.await?;
                                            continue 'con_loop;
                                        }
                                        _ => {
                                            Self::send_error_msg(
                                                net_send_closure,
                                                format!("Not a valid code {}", num),
                                            )
                                            .await?;
                                        }
                                    }
                                }
                            }
                        } else {
                            Self::send_error_msg(net_send_closure, "Without code").await?
                        }
                    }
                    Message::Binary(_) => todo!(),
                    Message::Ping(_) => {
                        net_sender.send(Message::Pong(vec![])).await?;
                    }
                    Message::Pong(_) => {
                        tracing::info!("recv pong");
                    }
                    Message::Frame(_) => todo!(),
                    Message::Close(_) => {
                        break 'con_loop;
                    }
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
                                    socket
                                        .send(Message::Text(serde_json::to_string(&resp).unwrap()))
                                        .await?;
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
        let request_sender = self.request_sender.clone();
        let mut shutdown_receiver = self.shutdown_sender.subscribe();
        let id;
        let mut http_sender = self.http_sender.take().unwrap();
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
                &request_sender,
                &mut http_sender,
                self.shared_data.clone(),
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
        Self::read_loop(
            incoming,
            id,
            msg_sender,
            request_sender,
            http_sender,
            shutdown_receiver,
            self.shared_data.clone(),
        )
        .await?;
        Ok(())
    }
}
