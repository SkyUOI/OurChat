//! 服务端

pub mod httpserver;
mod process;

use crate::connection;
use crate::connection::DBRequest;
use std::net::SocketAddr;
use std::process::exit;
use tokio::net::TcpStream;
use tokio::{
    net::TcpListener,
    select,
    sync::{broadcast, mpsc},
};

pub struct Server {
    addr: (String, u16),
    tcplistener: TcpListener,
    db: Option<sea_orm::DatabaseConnection>,
    redis: Option<redis::Client>,
    task_solver_sender: mpsc::Sender<DBRequest>,
    task_solver_receiver: Option<mpsc::Receiver<DBRequest>>,
    http_sender: mpsc::Sender<httpserver::Record>,
    test_mode: bool,
}

impl Server {
    pub async fn new(
        addr: (impl Into<String>, u16),
        db: sea_orm::DatabaseConnection,
        redis: redis::Client,
        http_sender: mpsc::Sender<httpserver::Record>,
        test_mode: bool,
    ) -> anyhow::Result<Self> {
        let ip = addr.0.into();
        let bind_addr = format!("{}:{}", &ip, addr.1);
        let tcplistener = match TcpListener::bind(&bind_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                tracing::error!("Failed to bind {}:{}", bind_addr, e);
                exit(1)
            }
        };
        let (task_solver_sender, task_solver_receiver) = mpsc::channel(32);
        let ret = Self {
            addr: (ip, addr.1),
            tcplistener,
            db: Some(db),
            redis: Some(redis),
            task_solver_sender,
            task_solver_receiver: Some(task_solver_receiver),
            test_mode,
            http_sender,
        };
        Ok(ret)
    }

    pub async fn accept_sockets(
        &mut self,
        shutdown_sender: broadcast::Sender<()>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        let db_coon = self.db.take().unwrap();
        let mut task_solver_receiver = self.task_solver_receiver.take().unwrap();
        tokio::spawn(async move {
            Self::process_db_request(&mut task_solver_receiver, &db_coon).await;
            (task_solver_receiver, db_coon)
        });
        let shutdown_sender_clone = shutdown_sender.clone();
        let async_loop = async move {
            loop {
                let task_sender = self.task_solver_sender.clone();
                let ret = self.tcplistener.accept().await;
                let shutdown_handle = shutdown_sender_clone.clone();
                let http_sender = self.http_sender.clone();
                match ret {
                    Ok((socket, addr)) => {
                        tracing::info!("Connected to a socket");
                        tokio::spawn(Server::handle_connection(
                            socket,
                            addr,
                            http_sender,
                            shutdown_handle,
                            task_sender,
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

    async fn process_db_request(
        receiver: &mut mpsc::Receiver<DBRequest>,
        db_connection: &sea_orm::DatabaseConnection,
    ) {
        while let Some(request) = receiver.recv().await {
            match match request {
                DBRequest::Login { resp, request } => {
                    Self::login(request, resp, db_connection).await
                }
                DBRequest::Register { resp, request } => {
                    Self::register(request, resp, db_connection).await
                }
                DBRequest::Unregister { id, resp } => {
                    Self::unregister(id, resp, db_connection).await
                }
                DBRequest::NewSession { id, resp } => {
                    Self::new_session(id, resp, db_connection).await
                }
                DBRequest::UpLoad { id, sz, resp } => {
                    Self::up_load(id, sz, resp, db_connection).await
                }
            } {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Database error:{e}");
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        http_sender: mpsc::Sender<httpserver::Record>,
        shutdown_sender: broadcast::Sender<()>,
        task_sender: mpsc::Sender<DBRequest>,
    ) {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Error during websocket handshake: {}", e);
                return;
            }
        };
        let mut connection =
            connection::Connection::new(ws_stream, http_sender, shutdown_sender, task_sender);
        match connection.work().await {
            Ok(_) => {
                tracing::info!("Connection closed: {}", addr);
            }
            Err(e) => {
                tracing::error!("Connection error: {}", e);
            }
        }
    }

    pub async fn delete(mut self) -> anyhow::Result<()> {
        if let Some(db) = self.db.take() {
            db.close().await?;
        }
        Ok(())
    }
}
