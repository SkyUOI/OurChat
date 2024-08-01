//! 服务端

use crate::connection::client_response;
use crate::connection::client_response::LoginResponse;
use crate::connection::client_response::RegisterResponse;
use crate::consts::ID;
use crate::{connection, consts, utils};
use crate::{connection::DBRequest, entities, requests};
use entities::user::ActiveModel as UserModel;
use sea_orm::*;
use snowdon::ClassicLayoutSnowflakeExtension;
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
    mysql: Option<sea_orm::DatabaseConnection>,
    redis: Option<redis::Client>,
    task_solver_sender: mpsc::Sender<DBRequest>,
    task_solver_receiver: Option<mpsc::Receiver<DBRequest>>,
    test_mode: bool,
}

impl Server {
    pub async fn new(
        ip: impl Into<String>,
        port: usize,
        mysql: sea_orm::DatabaseConnection,
        redis: redis::Client,
        test_mode: bool,
    ) -> anyhow::Result<Self> {
        let ip = ip.into();
        let bind_addr = format!("{}:{}", ip.clone(), port);
        let tcplistener = match TcpListener::bind(&bind_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                log::error!("Failed to bind {}:{}", bind_addr, e);
                exit(1)
            }
        };
        let (task_solver_sender, task_solver_receiver) = mpsc::channel(32);
        let ret = Self {
            ip: ip.clone(),
            port,
            bind_addr,
            tcplistener,
            mysql: Some(mysql),
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
            self.mysql.take().unwrap(),
        ));
        let async_loop = async move {
            loop {
                let task_sender = self.task_solver_sender.clone();
                let ret = self.tcplistener.accept().await;
                match ret {
                    Ok((socket, addr)) => {
                        let shutdown = shutdown_sender.subscribe();
                        log::info!("Connected to a socket");
                        tokio::spawn(async move {
                            Server::handle_connection(socket, addr, shutdown, task_sender).await
                        });
                    }
                    Err(_) => todo!(),
                }
            }
        };
        select! {
            _ = async_loop => {},
            _ = shutdown_receiver.recv() => {
                log::info!("Accepting loop exited")
            }
        }
    }

    async fn process_db_request(
        mut receiver: mpsc::Receiver<DBRequest>,
        mysql_connection: sea_orm::DatabaseConnection,
    ) {
        while let Some(request) = receiver.recv().await {
            match request {
                DBRequest::Login { resp, request } => {
                    use client_response::login::Status;
                    use entities::user::*;
                    // 判断帐号类型
                    let user = match request.login_type {
                        requests::LoginType::Email => {
                            Entity::find()
                                .filter(Column::Email.eq(request.account))
                                .one(&mysql_connection)
                                .await
                        }
                        requests::LoginType::Ocid => {
                            Entity::find()
                                .filter(Column::Ocid.eq(request.account))
                                .one(&mysql_connection)
                                .await
                        }
                    };
                    match user {
                        Ok(data) => match data {
                            Some(user) => {
                                if user.passwd == request.password {
                                    resp.send(Ok((LoginResponse::success(user.ocid), user.id)))
                                        .unwrap()
                                } else {
                                    resp.send(Err(Status::WrongPassword)).unwrap()
                                }
                            }
                            None => resp.send(Err(Status::WrongPassword)).unwrap(),
                        },
                        Err(e) => {
                            if let DbErr::RecordNotFound(_) = e {
                                resp.send(Err(Status::WrongPassword)).unwrap()
                            } else {
                                log::error!("database error:{}", e);
                                resp.send(Err(Status::ServerError)).unwrap()
                            }
                        }
                    }
                }
                DBRequest::Register { resp, request } => {
                    // 生成雪花id
                    let id = utils::generator().generate().unwrap().into_i64() as ID;
                    // 随机生成生成ocid
                    let ocid = utils::generate_ocid(consts::OCID_LEN);
                    let user = UserModel {
                        id: sea_orm::ActiveValue::Set(id),
                        ocid: sea_orm::ActiveValue::Set(ocid),
                        passwd: sea_orm::ActiveValue::Set(request.password),
                        name: sea_orm::ActiveValue::Set(request.name),
                        email: sea_orm::ActiveValue::Set(request.email),
                        time: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp() as u64),
                    };
                    match user.insert(&mysql_connection).await {
                        Ok(res) => {
                            // 生成正确的响应
                            let response = RegisterResponse::success(res.ocid);
                            resp.send(Ok((response, res.id))).unwrap();
                        }
                        Err(e) => {
                            if let sea_orm::DbErr::RecordNotInserted = e {
                                resp.send(Err(client_response::register::Status::Dup))
                                    .unwrap();
                            } else {
                                log::error!("Database error:{e}");
                                resp.send(Err(client_response::register::Status::ServerError))
                                    .unwrap();
                            }
                        }
                    }
                }
                DBRequest::Unregister { id, resp } => {
                    let user = UserModel {
                        id: ActiveValue::Set(id),
                        ..Default::default()
                    };
                    use client_response::unregister::Status;
                    match user.delete(&mysql_connection).await {
                        Ok(_) => resp.send(Status::Success).unwrap(),
                        Err(e) => {
                            log::error!("Database error:{e}");
                            resp.send(Status::Failed).unwrap();
                        }
                    }
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        shutdown_receiver: broadcast::Receiver<()>,
        task_sender: mpsc::Sender<DBRequest>,
    ) {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(data) => data,
            Err(e) => {
                log::error!("Error during websocket handshake: {}", e);
                return;
            }
        };
        tokio::spawn(async move {
            let mut connection =
                connection::Connection::new(ws_stream, shutdown_receiver, task_sender);
            match connection.work().await {
                Ok(_) => {
                    log::info!("Connection closed: {}", addr);
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        });
    }
}
