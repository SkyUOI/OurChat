//! Ourchat Server

pub mod httpserver;

use crate::component::EmailSender;
use crate::{DbPool, HttpSender, SharedData, connection};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::{net::TcpListener, select, sync::broadcast};
pub struct Server<T: EmailSender> {
    tcplistener: TcpListener,
    db: Option<DbPool>,
    http_sender: HttpSender,
    shared_data: Arc<SharedData<T>>,
}

impl<T: EmailSender> Server<T> {
    pub async fn new(
        tcplistener: TcpListener,
        db: DbPool,
        http_sender: HttpSender,
        shared_data: Arc<SharedData<T>>,
    ) -> anyhow::Result<Self> {
        let ret = Self {
            tcplistener,
            db: Some(db),
            http_sender,
            shared_data,
        };
        Ok(ret)
    }

    pub async fn accept_sockets(
        &mut self,
        shutdown_sender: broadcast::Sender<()>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        let db_conn = self.db.take().unwrap();

        let shutdown_sender_clone = shutdown_sender.clone();
        let async_loop = async move {
            loop {
                let ret = self.tcplistener.accept().await;
                let shutdown_handle = shutdown_sender_clone.clone();
                let http_sender = self.http_sender.clone();
                let db_conn = db_conn.clone();
                match ret {
                    Ok((socket, addr)) => {
                        tracing::info!("Connected to a socket");
                        tokio::spawn(Self::handle_connection(
                            socket,
                            addr,
                            http_sender,
                            shutdown_handle,
                            self.shared_data.clone(),
                            db_conn,
                        ));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to accept a socket: {}", e);
                    }
                }
            }
        };
        select! {
            _ = async_loop => {},
            _ = shutdown_receiver.recv() => {
                tracing::info!("Accepting loop exited")
            }
        }
        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        http_sender: HttpSender,
        shutdown_sender: broadcast::Sender<()>,
        shared_data: Arc<SharedData<T>>,
        dbpool: DbPool,
    ) {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Error during websocket handshake: {}", e);
                return;
            }
        };
        let mut connection = connection::Connection::new(
            ws_stream,
            http_sender,
            shutdown_sender,
            shared_data,
            dbpool,
        );
        match connection.work().await {
            Ok(_) => {
                tracing::info!("Connection closed: {}", addr);
            }
            Err(e) => {
                tracing::error!("Connection error: {}", crate::utils::error_chain(e));
            }
        }
    }
}
