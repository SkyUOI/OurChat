mod cfg;
mod connection;
pub mod consts;
mod db;
mod entities;
pub mod requests;
mod utils;

use clap::Parser;
use connection::{
    client_response::{self, register::RegisterResponse},
    response::{LoginError, RegisterError},
    DBRequest,
};
use entities::user::ActiveModel as UserModel;
use rand::Rng;
use requests::Register;
use sea_orm::ActiveModelTrait;
use serde::{Deserialize, Serialize};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::{fs, io::Write, net::SocketAddr, path, process::exit, sync::OnceLock};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
    select,
    sync::{broadcast, mpsc},
};

shadow_rs::shadow!(build);

type ShutdownRev = broadcast::Receiver<()>;

#[derive(Debug, Parser)]
#[command(author = "SkyUOI", version = build::VERSION, about = "The Server of OurChat")]
struct ArgsParser {
    #[arg(short, long, default_value_t = cfg::DEFAULT_PORT)]
    port: usize,
    #[arg(long, default_value_t = String::from(cfg::DEFAULT_IP))]
    ip: String,
    #[arg(long)]
    cfg: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cfg {
    rediscfg: String,
    dbcfg: String,
}

struct Server {
    ip: String,
    port: usize,
    bind_addr: String,
    tcplistener: TcpListener,
    mysql: Option<sea_orm::DatabaseConnection>,
    redis: Option<redis::Client>,
    task_solver_sender: mpsc::Sender<DBRequest>,
    task_solver_receiver: Option<mpsc::Receiver<DBRequest>>,
}

impl Server {
    pub async fn new(
        ip: impl Into<String>,
        port: usize,
        mysql: sea_orm::DatabaseConnection,
        redis: redis::Client,
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
        };
        Ok(ret)
    }

    async fn accept_sockets(
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
                DBRequest::Login { resp, request } => {}
                DBRequest::Register { resp, request } => {
                    // 生成雪花id
                    let id = utils::generator().generate().unwrap().into_i64() as u64;
                    // 随机生成生成ocid
                    let ocid = utils::generate_ocid(consts::OCID_LEN);
                    let user = UserModel {
                        id: sea_orm::ActiveValue::Set(id),
                        ocid: sea_orm::ActiveValue::Set(ocid),
                        passwd: sea_orm::ActiveValue::Set(request.password),
                        name: sea_orm::ActiveValue::Set(request.name),
                        email: sea_orm::ActiveValue::Set(request.email),
                        time: sea_orm::ActiveValue::Set(request.time),
                    };
                    match user.insert(&mysql_connection).await {
                        Ok(res) => {
                            // 生成正确的响应
                            let response = RegisterResponse::new(
                                res.ocid,
                                client_response::register::Status::Success,
                            );
                            resp.send(Ok(response)).unwrap();
                        }
                        Err(e) => {
                            if let sea_orm::DbErr::RecordNotInserted = e {
                                resp.send(Err(RegisterError::AlreadyExists)).unwrap();
                            } else {
                                log::error!("Database error:{e}");
                                resp.send(Err(RegisterError::ServerError(e.to_string())))
                                    .unwrap();
                            }
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

fn machine_id() -> u64 {
    static TMP: OnceLock<u64> = OnceLock::new();
    *TMP.get_or_init(|| {
        let state = path::Path::new("machine_id").exists();
        if state {
            return u64::from_be_bytes(fs::read("machine_id").unwrap().try_into().unwrap());
        }
        let mut f = fs::File::create("machine_id").unwrap();
        let id: u64 = rand::thread_rng().gen_range(0..(1024 - 1));
        f.write_all(&id.to_be_bytes()).unwrap();
        id
    })
}

pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let cfg_path = parser.cfg;
    let cfg = fs::read_to_string(cfg_path)?;
    let cfg: Cfg = toml::from_str(&cfg).unwrap();
    let port = parser.port;
    let ip = parser.ip;
    // 用于通知关闭的channel
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(32);
    let shutdown_sender_clone = shutdown_sender.clone();
    let shutdown_receiver_clone = shutdown_sender.subscribe();

    let db = db::connect_to_db(&db::get_db_url(&cfg.dbcfg)?).await?;
    db::init_db(&db).await?;
    let redis = db::connect_to_redis(&db::get_redis_url(&cfg.rediscfg)?).await?;
    let mut server = Server::new(ip, port, db, redis).await?;
    tokio::spawn(async move {
        server
            .accept_sockets(shutdown_sender_clone, shutdown_receiver_clone)
            .await
    });
    let shutdown_sender_clone = shutdown_sender.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                log::info!("Exiting now...");
                shutdown_sender_clone.send(())?;
            }
            Err(err) => {
                log::error!("Unable to listen to ctrl-c signal:{}", err);
                shutdown_sender_clone.send(())?;
            }
        }
        anyhow::Ok(())
    });
    let mut console_reader = BufReader::new(io::stdin()).lines();
    let input_loop = async {
        loop {
            print!(">>>");
            std::io::stdout().flush().unwrap();
            let command = match console_reader.next_line().await {
                Ok(d) => match d {
                    Some(data) => data,
                    None => break,
                },
                Err(e) => {
                    log::error!("{}", e);
                    break;
                }
            };
            let command = command.trim();
            if command == "exit" {
                log::info!("Exiting now...");
                break;
            }
        }
        anyhow::Ok(())
    };
    select! {
        _ = input_loop => {
            shutdown_sender.send(())?;
        },
        _ = shutdown_receiver.recv() => {
            log::info!("Command loop exited");
        }
    }
    Ok(())
}
