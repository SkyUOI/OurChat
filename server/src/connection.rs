use futures_util::StreamExt;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::WebSocketStream;

use crate::ShutdownRev;

pub enum Request {}

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

    async fn login() {}

    pub async fn work(&mut self) -> anyhow::Result<()> {
        let socket = self.socket.take().unwrap();
        let (mut outgoing, mut incoming) = socket.split();
        let msg = match incoming.next().await {
            Some(Ok(msg)) => msg,
            _ => {
                return Err(anyhow::anyhow!("Failed to receive message when logining"));
            }
        };
        Ok(())
    }
}
