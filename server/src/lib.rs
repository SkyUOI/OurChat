mod cfg;
use clap::Parser;
use std::{io::Write, process::exit};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::TcpListener,
    select,
    sync::broadcast,
};

shadow_rs::shadow!(build);

#[derive(Debug, Parser)]
#[command(author = "SkyUOI", version = build::VERSION, about = "The Server of OurChat")]
struct ArgsParser {
    #[arg(short, long, default_value_t = cfg::DEFAULT_PORT)]
    port: usize,
    #[arg(long, default_value_t = String::from(cfg::DEFAULT_IP))]
    ip: String,
}

struct Server {
    ip: String,
    port: usize,
    bind_addr: String,
    tcplistener: TcpListener,
}

impl Server {
    pub async fn new(ip: impl Into<String>, port: usize) -> anyhow::Result<Self> {
        let ip = ip.into();
        let bind_addr = format!("{}:{}", ip.clone(), port);
        let tcplistener = match TcpListener::bind(&bind_addr).await {
            Ok(listener) => listener,
            Err(e) => {
                log::error!("Failed to bind {}:{}", bind_addr, e);
                exit(1)
            }
        };
        Ok(Self {
            ip: ip.clone(),
            port,
            bind_addr,
            tcplistener,
        })
    }

    async fn accept_sockets(
        &mut self,
        shutdown_sender: broadcast::Sender<()>,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) {
        let async_loop = async move {
            loop {
                let ret = self.tcplistener.accept().await;
                match ret {
                    Ok((socket, _)) => {
                        let shutdown = shutdown_sender.subscribe();
                        log::info!("Connected to a socket");
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
}

pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    let port = parser.port;
    let ip = parser.ip;
    // 用于通知关闭的channel
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(32);
    let shutdown_sender_clone = shutdown_sender.clone();
    let shutdown_receiver_clone = shutdown_sender.subscribe();
    let mut server = Server::new(ip, port).await?;
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
