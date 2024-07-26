mod cfg;
mod connection;

use clap::Parser;
use connection::Request;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::Connection;
use std::{io::Write, net::SocketAddr, process::exit};
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
    dbcfg: String,
}

struct Server {
    ip: String,
    port: usize,
    bind_addr: String,
    tcplistener: TcpListener,
    mysql: Option<sqlx::MySqlConnection>,
    task_solver_sender: mpsc::Sender<Request>,
    task_solver_receiver: Option<mpsc::Receiver<Request>>,
}

impl Server {
    pub async fn new(
        ip: impl Into<String>,
        port: usize,
        mysql: sqlx::MySqlConnection,
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
        tokio::spawn(Self::process_request(
            self.task_solver_receiver.take().unwrap(),
            self.mysql.take().unwrap(),
        ));
        let async_loop = async move {
            loop {
                let ret = self.tcplistener.accept().await;
                match ret {
                    Ok((socket, addr)) => {
                        let shutdown = shutdown_sender.subscribe();
                        log::info!("Connected to a socket");
                        tokio::spawn(async move {
                            Server::handle_connection(socket, addr, shutdown).await
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

    async fn process_request(
        mut receiver: mpsc::Receiver<Request>,
        mysql_connection: sqlx::MySqlConnection,
    ) {
        while let Some(request) = receiver.recv().await {
            match request {}
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        shutdown_receiver: broadcast::Receiver<()>,
    ) {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(data) => data,
            Err(e) => {
                log::error!("Error during websocket handshake: {}", e);
                return;
            }
        };
        tokio::spawn(async move {
            let mut connection = connection::Connection::new(ws_stream, shutdown_receiver);
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

#[derive(Debug, Deserialize, Serialize)]
struct DbCfg {
    host: String,
    user: String,
    db: String,
    port: usize,
    passwd: String,
}

async fn connect_to_db(path: &str) -> anyhow::Result<sqlx::MySqlConnection> {
    let json = std::fs::read_to_string(path)?;
    let cfg: DbCfg = serde_json::from_str(&json)?;
    let path = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.user, cfg.passwd, cfg.host, cfg.port, cfg.db
    );
    Ok(sqlx::MySqlConnection::connect(&path).await?)
}

pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let port = parser.port;
    let ip = parser.ip;
    // 用于通知关闭的channel
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(32);
    let shutdown_sender_clone = shutdown_sender.clone();
    let shutdown_receiver_clone = shutdown_sender.subscribe();
    let db = connect_to_db(&parser.dbcfg).await?;
    let mut server = Server::new(ip, port, db).await?;
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