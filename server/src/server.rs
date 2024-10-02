//! 服务端

pub mod httpserver;
mod process;

use crate::component::EmailSender;
use crate::connection::DBRequest;
use crate::{HttpSender, SharedData, connection};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::{
    net::TcpListener,
    select,
    sync::{broadcast, mpsc},
};
pub struct Server<T: EmailSender> {
    tcplistener: TcpListener,
    db: Option<sea_orm::DatabaseConnection>,
    _redis: Option<redis::Client>,
    task_solver_sender: mpsc::Sender<DBRequest>,
    task_solver_receiver: Option<mpsc::Receiver<DBRequest>>,
    http_sender: HttpSender,
    shared_data: Arc<SharedData<T>>,
}

impl<T: EmailSender> Server<T> {
    pub async fn new(
        tcplistener: TcpListener,
        db: sea_orm::DatabaseConnection,
        redis: redis::Client,
        http_sender: HttpSender,
        shared_data: Arc<SharedData<T>>,
    ) -> anyhow::Result<Self> {
        let (task_solver_sender, task_solver_receiver) = mpsc::channel(32);
        let ret = Self {
            tcplistener,
            db: Some(db),
            _redis: Some(redis),
            task_solver_sender,
            task_solver_receiver: Some(task_solver_receiver),
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
                        tokio::spawn(Self::handle_connection(
                            socket,
                            addr,
                            http_sender,
                            shutdown_handle,
                            task_sender,
                            self.shared_data.clone(),
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
        http_sender: HttpSender,
        shutdown_sender: broadcast::Sender<()>,
        task_sender: mpsc::Sender<DBRequest>,
        shared_data: Arc<SharedData<T>>,
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
            task_sender,
            shared_data,
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

    pub async fn delete(mut self) -> anyhow::Result<()> {
        if let Some(db) = self.db.take() {
            db.close().await?;
        }
        Ok(())
    }
}
