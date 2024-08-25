//! 服务端

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
    ip: String,
    port: usize,
    bind_addr: String,
    tcplistener: TcpListener,
    db: Option<sea_orm::DatabaseConnection>,
    redis: Option<redis::Client>,
    task_solver_sender: mpsc::Sender<DBRequest>,
    task_solver_receiver: Option<mpsc::Receiver<DBRequest>>,
    test_mode: bool,
}

impl Server {
    pub async fn new(
        ip: impl Into<String>,
        port: usize,
        db: sea_orm::DatabaseConnection,
        redis: redis::Client,
        test_mode: bool,
    ) -> anyhow::Result<Self> {
        let ip = ip.into();
        let bind_addr = format!("{}:{}", ip.clone(), port);
        let tcplistener = match TcpListener::bind(&bind_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                tracing::error!("Failed to bind {}:{}", bind_addr, e);
                exit(1)
            }
        };
        let (task_solver_sender, task_solver_receiver) = mpsc::channel(32);
        let ret = Self {
            ip: ip.clone(),
            port,
            bind_addr,
            tcplistener,
            db: Some(db),
            redis: Some(redis),
            task_solver_sender,
            task_solver_receiver: Some(task_solver_receiver),
            test_mode,
        };
        Ok(ret)
    }

    pub async fn accept_sockets(
        &mut self,
        shutdown_sender: broadcast::Sender<()>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) {
        tokio::spawn(Self::process_db_request(
            self.task_solver_receiver.take().unwrap(),
            self.db.take().unwrap(),
        ));
        let shutdown_sender_clone = shutdown_sender.clone();
        let async_loop = async move {
            loop {
                let task_sender = self.task_solver_sender.clone();
                let ret = self.tcplistener.accept().await;
                let shutdown_handle = shutdown_sender_clone.clone();
                match ret {
                    Ok((socket, addr)) => {
                        tracing::info!("Connected to a socket");
                        tokio::spawn(async move {
                            Server::handle_connection(socket, addr, shutdown_handle, task_sender)
                                .await
                        });
                    }
                    Err(_) => todo!(),
                }
            }
        };
        select! {
            _ = async_loop => {},
            _ = shutdown_receiver.recv() => {
                tracing::info!("Accepting loop exited")
            }
        }
    }

    async fn process_db_request(
        mut receiver: mpsc::Receiver<DBRequest>,
        db_connection: sea_orm::DatabaseConnection,
    ) {
        while let Some(request) = receiver.recv().await {
            match request {
                DBRequest::Login { resp, request } => {
                    Self::login(request, resp, &db_connection).await
                }
                DBRequest::Register { resp, request } => {
                    Self::register(request, resp, &db_connection).await;
                }
                DBRequest::Unregister { id, resp } => {
                    Self::unregister(id, resp, &db_connection).await;
                }
                DBRequest::NewSession { id, resp } => {
                    Self::new_session(id, resp, &db_connection).await;
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
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
        tokio::spawn(async move {
            let mut connection =
                connection::Connection::new(ws_stream, shutdown_sender, task_sender);
            match connection.work().await {
                Ok(_) => {
                    tracing::info!("Connection closed: {}", addr);
                }
                Err(e) => {
                    tracing::error!("Connection error: {}", e);
                }
            }
        });
    }
}
