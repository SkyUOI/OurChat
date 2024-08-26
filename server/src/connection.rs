//! 处理到客户端的连接

mod basic;
pub mod client_response;
mod process;

use crate::{
    consts::{MessageType, ID},
    requests::{self, new_session::NewSession},
    MAINTAINING,
};
use anyhow::bail;
use client_response::{
    get_status::GetStatusResponse, LoginResponse, NewSessionResponse, RegisterResponse,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::Value;
use static_keys::static_branch_unlikely;
use tokio::{
    net::TcpStream,
    select,
    sync::{broadcast, mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

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
}

pub type WS = WebSocketStream<TcpStream>;

/// 一个到客户端的连接
pub struct Connection {
    socket: Option<WebSocketStream<TcpStream>>,
    shutdown_sender: broadcast::Sender<()>,
    request_sender: mpsc::Sender<DBRequest>,
}
enum VerifyStatus {
    Success,
    Fail,
}

struct VerifyRes {
    status: VerifyStatus,
    id: ID,
}

impl Connection {
    pub fn new(
        socket: WS,
        shutdown_sender: broadcast::Sender<()>,
        request_sender: mpsc::Sender<DBRequest>,
    ) -> Self {
        Self {
            socket: Some(socket),
            shutdown_sender,
            request_sender,
        }
    }

    /// 登录请求
    async fn login_request(
        request_sender: &mpsc::Sender<DBRequest>,
        login_data: requests::Login,
    ) -> anyhow::Result<(String, VerifyRes)> {
        let channel = oneshot::channel();
        let request = DBRequest::Login {
            request: login_data,
            resp: channel.0,
        };
        request_sender.send(request).await?;
        match channel.1.await? {
            Ok(ok_resp) => Ok((
                serde_json::to_string(&ok_resp.0).unwrap(),
                VerifyRes {
                    status: VerifyStatus::Success,
                    id: ok_resp.1,
                },
            )),
            Err(e) => Ok((
                serde_json::to_string(&LoginResponse::failed(e)).unwrap(),
                VerifyRes {
                    status: VerifyStatus::Fail,
                    id: ID::default(),
                },
            )),
        }
    }

    /// 注册请求
    async fn register_request(
        request_sender: &mpsc::Sender<DBRequest>,
        register_data: requests::Register,
    ) -> anyhow::Result<(String, VerifyRes)> {
        let channel = oneshot::channel();
        let request = DBRequest::Register {
            request: register_data,
            resp: channel.0,
        };
        request_sender.send(request).await?;
        match channel.1.await? {
            Ok(ok_resp) => Ok((
                serde_json::to_string(&ok_resp.0).unwrap(),
                VerifyRes {
                    status: VerifyStatus::Success,
                    id: ok_resp.1,
                },
            )),
            Err(e) => Ok((
                serde_json::to_string(&RegisterResponse::failed(e)).unwrap(),
                VerifyRes {
                    status: VerifyStatus::Fail,
                    id: ID::default(),
                },
            )),
        }
    }

    /// 验证客户端
    /// 注意：我们允许一些级别很低的非验证操作在此阶段运行
    async fn verify(
        ws: &mut WS,
        request_sender: &mpsc::Sender<DBRequest>,
    ) -> anyhow::Result<Option<(String, VerifyRes)>> {
        loop {
            let msg = match ws.next().await {
                None => {
                    anyhow::bail!("Failed to receive message when logining");
                }
                Some(res) => match res {
                    Ok(msg) => msg,
                    Err(e) => Err(e)?,
                },
            };
            match msg {
                Message::Text(text) => {
                    let json: Value = serde_json::from_str(&text)?;
                    // 获取消息类型
                    let code = &json["code"];
                    if let Value::Number(code) = code {
                        let code = code.as_u64();
                        if code == Some(MessageType::Login as u64) {
                            let login_data: requests::Login = serde_json::from_value(json)?;
                            let resp = Self::login_request(request_sender, login_data).await?;
                            return Ok(Some(resp));
                        } else if code == Some(MessageType::Register as u64) {
                            let request_data: requests::Register = serde_json::from_value(json)?;
                            let resp = Self::register_request(request_sender, request_data).await?;
                            return Ok(Some(resp));
                        } else if code == Some(MessageType::GetStatus as u64) {
                            let resp = GetStatusResponse::normal();
                            let resp = serde_json::to_string(&resp)?;
                            ws.send(Message::Text(resp)).await?;
                            continue;
                        } else {
                            // 验证不通过
                            let resp = serde_json::to_string(
                                &client_response::error_msg::ErrorMsgResponse::new(
                                    "Not login or register code".to_string(),
                                ),
                            )?;
                            tracing::info!(
                                "Failed to login,code is {:?},not login or register code",
                                code
                            );
                            return Ok(Some((
                                resp,
                                VerifyRes {
                                    status: VerifyStatus::Fail,
                                    id: ID::default(),
                                },
                            )));
                        }
                    } else {
                        anyhow::bail!("Failed to login,code is not a number or missing");
                    }
                }
                Message::Close(_) => {
                    return Ok(None);
                }
                _ => {
                    anyhow::bail!("Failed to login,not a text message");
                }
            }
        }
    }

    pub async fn read_loop(
        mut incoming: SplitStream<WS>,
        id: ID,
        net_sender: mpsc::Sender<Message>,
        request_sender: mpsc::Sender<DBRequest>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
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
                    Err(e) => {
                        tracing::error!("recv error:{}", e);
                        Err(e)?
                    }
                };
                match msg {
                    tungstenite::Message::Text(msg) => {
                        let json: Value = serde_json::from_str(&msg)?;
                        let code = &json["code"];
                        if let Value::Number(code) = code {
                            let code = code.as_u64();
                            match code {
                                None => {
                                    Self::send_error_msg(
                                        &net_sender,
                                        "code is not a unsigned number",
                                    )
                                    .await?;
                                }
                                Some(num) => {
                                    let code = match MessageType::try_from(num as i32) {
                                        Ok(num) => num,
                                        Err(_) => {
                                            Self::send_error_msg(
                                                &net_sender,
                                                format!("Not a valid code {}", num),
                                            )
                                            .await?;
                                            continue 'con_loop;
                                        }
                                    };
                                    match code {
                                        MessageType::Unregister => {
                                            Self::unregister(id, &request_sender, &net_sender)
                                                .await?;
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
                                            Self::new_session(
                                                id,
                                                &request_sender,
                                                &net_sender,
                                                json,
                                            )
                                            .await?;
                                            continue 'con_loop;
                                        }
                                        _ => {
                                            Self::send_error_msg(
                                                &net_sender,
                                                format!("Not a valid code {}", num),
                                            )
                                            .await?;
                                        }
                                    }
                                }
                            }
                        } else {
                            Self::send_error_msg(&net_sender, "Without code").await?
                        }
                    }
                    tungstenite::Message::Binary(_) => todo!(),
                    tungstenite::Message::Ping(_) => {
                        net_sender.send(Message::Pong(vec![])).await?;
                    }
                    tungstenite::Message::Pong(_) => {
                        tracing::info!("recv pong");
                    }
                    tungstenite::Message::Frame(_) => todo!(),
                    tungstenite::Message::Close(_) => {
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
        mut outgoing: SplitSink<WS, Message>,
        mut receiver: mpsc::Receiver<Message>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        let work = async {
            while let Some(msg) = receiver.recv().await {
                tracing::debug!("send msg:{}", msg);
                outgoing.send(msg).await.unwrap();
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
                    tungstenite::Message::Text(msg) => {
                        let json: Value = serde_json::from_str(&msg)?;
                        let code = &json["code"];
                        if let Value::Number(code) = code {
                            let code = code.as_u64();
                            if let Some(code) = code {
                                if code == MessageType::GetStatus as u64 {
                                    let resp = GetStatusResponse::mataining();
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
                    tungstenite::Message::Ping(_) => {
                        socket.send(Message::Pong(vec![])).await?;
                    }
                    tungstenite::Message::Pong(_) => {
                        tracing::info!("recv pong");
                    }
                    _ => {}
                },
                Err(e) => Err(e)?,
            };
        }
        Ok(())
    }

    pub async fn work(&mut self) -> anyhow::Result<()> {
        let mut socket = self.socket.take().unwrap();
        let request_sender = self.request_sender.clone();
        let mut shutdown_receiver = self.shutdown_sender.subscribe();
        let mut shutdown_receiver_clone = self.shutdown_sender.subscribe();
        let mut id = ID::default();

        if static_branch_unlikely!(MAINTAINING) {
            Self::maintaining(&mut socket).await?;
            return Ok(());
        }
        let verify_loop = async {
            loop {
                let ret = Connection::verify(&mut socket, &request_sender).await?;
                match ret {
                    None => {
                        return anyhow::Ok(());
                    }
                    Some(ret) => {
                        select! {
                            err = socket.send(tungstenite::Message::Text(ret.0)) => {
                                err?
                            },
                            _ = shutdown_receiver.recv() => {
                                return anyhow::Ok(())
                            },
                        }
                        if let VerifyStatus::Success = ret.1.status {
                            // 验证通过，跳出循环
                            id = ret.1.id;
                            break;
                        }
                    }
                }
            }
            Ok(())
        };
        // 循环验证直到验证通过
        select! {
            ret = verify_loop => {
                ret?
            },
            _ = shutdown_receiver_clone.recv() => {
                return Ok(());
            },
        }
        let (outgoing, incoming) = socket.split();
        let (sender, receiver) = mpsc::channel(32);
        tokio::spawn(Self::write_loop(
            outgoing,
            receiver,
            self.shutdown_sender.subscribe(),
        ));
        Self::read_loop(incoming, id, sender, request_sender, shutdown_receiver).await?;
        Ok(())
    }
}
