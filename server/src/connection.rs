use crate::{
    consts::{LOGIN_TYPE, REGISTER_TYPE},
    ShutdownRev,
};
use futures_util::StreamExt;
use serde_json::Value;
use tokio::{net::TcpStream, select};
use tokio_tungstenite::WebSocketStream;

pub enum Request {}

type WS = WebSocketStream<TcpStream>;

pub struct Connection {
    socket: Option<WebSocketStream<TcpStream>>,
    shutdown_receiver: ShutdownRev,
}

impl Connection {
    pub fn new(socket: WebSocketStream<TcpStream>, shutdown_receiver: ShutdownRev) -> Self {
        Self {
            socket: Some(socket),
            shutdown_receiver,
        }
    }

    async fn login(ws: &mut WS) -> anyhow::Result<()> {
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
        let json: serde_json::Value = serde_json::from_str(text)?;
        let code = &json["code"];
        if let Value::Number(code) = code {
            let code = code.as_u64();
            if code == Some(LOGIN_TYPE) {
            } else if code == Some(REGISTER_TYPE) {
            } else {
                anyhow::bail!(
                    "Failed to login,code is {:?},not login or register code",
                    code
                );
            }
        } else {
            anyhow::bail!("Failed to login,code is not a number or missing");
        }
        Ok(())
    }

    pub async fn work(&mut self) -> anyhow::Result<()> {
        let mut socket = self.socket.take().unwrap();
        select! {
            _ = Connection::login(&mut socket) => {},
            _ = self.shutdown_receiver.recv() => {},
        }
        Ok(())
    }
}
