pub mod client_response;
pub mod response;

use crate::{
    consts::RequestType,
    requests::{self},
    ShutdownRev,
};
use client_response::{login::LoginResponse, register::RegisterResponse};
use futures_util::{SinkExt, StreamExt};
use response::{LoginError, RegisterError};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    select,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;

pub enum DBRequest {
    Login {
        request: requests::Login,
        resp: oneshot::Sender<Result<LoginResponse, LoginError>>,
    },
    Register {
        request: requests::Register,
        resp: oneshot::Sender<Result<RegisterResponse, RegisterError>>,
    },
}

type WS = WebSocketStream<TcpStream>;

/// 一个到客户端的连接
pub struct Connection {
    socket: Option<WebSocketStream<TcpStream>>,
    shutdown_receiver: ShutdownRev,
    request_sender: mpsc::Sender<DBRequest>,
}

impl Connection {
    pub fn new(
        socket: WebSocketStream<TcpStream>,
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
    ) -> anyhow::Result<String> {
        let channel = oneshot::channel();
        let request = DBRequest::Login {
            request: login_data,
            resp: channel.0,
        };
        request_sender.send(request).await?;
        let data = serde_json::to_string(&channel.1.await??).unwrap();
        Ok(data)
    }

    /// 注册请求
    async fn register_request(
        request_sender: &mpsc::Sender<DBRequest>,
        register_data: requests::Register,
    ) -> anyhow::Result<String> {
        let channel = oneshot::channel();
        let request = DBRequest::Register {
            request: register_data,
            resp: channel.0,
        };
        request_sender.send(request).await?;
        let resp = serde_json::to_string(&channel.1.await??).unwrap();
        Ok(resp)
    }

    /// 验证客户端
    async fn verify(
        ws: &mut WS,
        request_sender: &mpsc::Sender<DBRequest>,
    ) -> anyhow::Result<String> {
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
            if code == Some(RequestType::Login as u64) {
                let login_data: requests::Login = serde_json::from_value(json)?;
                Self::login_request(request_sender, login_data).await?;
                return Ok(todo!());
            } else if code == Some(RequestType::Register as u64) {
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

    pub async fn work(&mut self) -> anyhow::Result<()> {
        let mut socket = self.socket.take().unwrap();
        let sender = self.request_sender.clone();
        select! {
            ret = Connection::verify(&mut socket, &sender) => {
                let ret = ret?;
                select! {
                    err = socket.send(tungstenite::Message::Text(ret)) => {
                        err?
                    },
                    _ = self.shutdown_receiver.recv() => {},
                }
            },
            _ = self.shutdown_receiver.recv() => {},
        }
        Ok(())
    }
}
