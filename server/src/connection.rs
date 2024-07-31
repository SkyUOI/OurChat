//! 处理到客户端的连接

pub mod client_response;

use crate::{
    consts::MessageType,
    requests::{self},
    ShutdownRev,
};
use client_response::{LoginResponse, RegisterResponse};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    select,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub enum DBRequest {
    Login {
        request: requests::Login,
        resp: oneshot::Sender<Result<LoginResponse, client_response::login::Status>>,
    },
    Register {
        request: requests::Register,
        resp: oneshot::Sender<Result<RegisterResponse, client_response::register::Status>>,
    },
}

pub type WS = WebSocketStream<TcpStream>;

/// 一个到客户端的连接
pub struct Connection {
    socket: Option<WebSocketStream<TcpStream>>,
    shutdown_receiver: ShutdownRev,
    request_sender: mpsc::Sender<DBRequest>,
}
enum VerifyStatus {
    Success,
    Fail,
}

impl Connection {
    pub fn new(
        socket: WS,
        shutdown_receiver: ShutdownRev,
        request_sender: mpsc::Sender<DBRequest>,
    ) -> Self {
        Self {
            socket: Some(socket),
            shutdown_receiver,
            request_sender,
        }
    }

    /// 登录请求
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
            Ok(ok_resp) => {
                return Ok((
                    serde_json::to_string(&ok_resp).unwrap(),
                    VerifyStatus::Success,
                ));
            }
            Err(e) => {
                return Ok((
                    serde_json::to_string(&LoginResponse::failed(e)).unwrap(),
                    VerifyStatus::Fail,
                ))
            }
        }
    }

    /// 注册请求
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
            Ok(ok_resp) => {
                return Ok((
                    serde_json::to_string(&ok_resp).unwrap(),
                    VerifyStatus::Success,
                ));
            }
            Err(e) => {
                return Ok((
                    serde_json::to_string(&RegisterResponse::failed(e)).unwrap(),
                    VerifyStatus::Fail,
                ))
            }
        }
    }

    /// 验证客户端
    async fn verify(
        ws: &mut WS,
        request_sender: &mpsc::Sender<DBRequest>,
    ) -> anyhow::Result<(String, VerifyStatus)> {
        let msg = match ws.next().await {
            None => {
                anyhow::bail!("Failed to receive message when logining");
            }
            Some(res) => match res {
                Ok(msg) => msg,
                Err(e) => Err(e)?,
            },
        };
        let text = match msg.to_text() {
            Ok(text) => text,
            Err(e) => anyhow::bail!("Failed to convert message to text: {}", e),
        };
        let json: Value = serde_json::from_str(text)?;
        // 获取消息类型
        let code = &json["code"];
        if let Value::Number(code) = code {
            let code = code.as_u64();
            if code == Some(MessageType::Login as u64) {
                let login_data: requests::Login = serde_json::from_value(json)?;
                let resp = Self::login_request(request_sender, login_data).await?;
                return Ok(resp);
            } else if code == Some(MessageType::Register as u64) {
                let request_data: requests::Register = serde_json::from_value(json)?;
                let resp = Self::register_request(request_sender, request_data).await?;
                return Ok(resp);
            } else {
                anyhow::bail!(
                    "Failed to login,code is {:?},not login or register code",
                    code
                );
            }
        } else {
            anyhow::bail!("Failed to login,code is not a number or missing");
        }
    }

    async fn send_error_msg(
        sender: &mpsc::Sender<Message>,
        msg: impl Into<String>,
    ) -> anyhow::Result<()> {
        let error_resp = client_response::error_msg::ErrorMsgResponse::new(msg.into());
        sender
            .send(Message::Text(serde_json::to_string(&error_resp)?))
            .await?;
        Ok(())
    }

    pub async fn read_loop(
        mut incoming: SplitStream<WS>,
        sender: mpsc::Sender<Message>,
        request_sender: mpsc::Sender<DBRequest>,
    ) -> anyhow::Result<()> {
        loop {
            let msg = incoming.next().await;
            if msg.is_none() {
                return Ok(());
            }
            let msg = msg.unwrap();
            let msg = match msg {
                Ok(msg) => {
                    log::debug!("recv msg:{}", msg);
                    msg
                }
                Err(e) => {
                    log::error!("recv error:{}", e);
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
                                Self::send_error_msg(&sender, "code is not a unsigned number")
                                    .await?;
                            }
                            Some(num) => {
                                let code = match MessageType::try_from(num as i32) {
                                    Ok(num) => num,
                                    Err(_) => {
                                        Self::send_error_msg(
                                            &sender,
                                            format!("Not a valid code {}", num),
                                        )
                                        .await?;
                                        return Ok(());
                                    }
                                };
                                match code {
                                    MessageType::Unregister => {
                                        // let unregister  =
                                    }
                                    _ => {
                                        Self::send_error_msg(
                                            &sender,
                                            format!("Not a valid code {}", num),
                                        )
                                        .await?;
                                    }
                                }
                            }
                        }
                    } else {
                        Self::send_error_msg(&sender, "Without code").await?
                    }
                }
                tungstenite::Message::Binary(_) => todo!(),
                tungstenite::Message::Ping(_) => {
                    sender.send(Message::Pong(vec![])).await?;
                }
                tungstenite::Message::Pong(_) => {
                    log::info!("recv pong");
                }
                tungstenite::Message::Close(_) => {
                    return Ok(());
                }
                tungstenite::Message::Frame(_) => todo!(),
            }
        }
    }

    pub async fn write_loop(
        mut outgoing: SplitSink<WS, Message>,
        receiver: mpsc::Receiver<Message>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn work(&mut self) -> anyhow::Result<()> {
        let mut socket = self.socket.take().unwrap();
        let request_sender = self.request_sender.clone();
        // 循环验证直到验证通过
        'verify: loop {
            select! {
                ret = Connection::verify(&mut socket, &request_sender) => {
                    let ret = ret?;
                    select! {
                        err = socket.send(tungstenite::Message::Text(ret.0)) => {
                            err?
                        },
                        _ = self.shutdown_receiver.recv() => {
                            return Ok(())
                        },
                    }
                    if let VerifyStatus::Success = ret.1 {
                        // 验证通过，跳出循环
                        break 'verify;
                    }
                },
                _ = self.shutdown_receiver.recv() => {
                    return Ok(());
                },
            }
        }
        let (outgoing, incoming) = socket.split();
        let (sender, receiver) = mpsc::channel(32);
        Self::read_loop(incoming, sender, request_sender).await?;
        Self::write_loop(outgoing, receiver).await?;
        Ok(())
    }
}
