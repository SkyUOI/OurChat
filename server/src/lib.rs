pub mod connection;
pub mod consts;
mod db;
mod entities;
pub mod requests;
mod server;
pub mod utils;

use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path, sync::OnceLock};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    select,
    sync::broadcast,
};

shadow_rs::shadow!(build);

type ShutdownRev = broadcast::Receiver<()>;

#[derive(Debug, Parser)]
#[command(author = "SkyUOI", version = build::VERSION, about = "The Server of OurChat")]
struct ArgsParser {
    #[arg(short, long, default_value_t = consts::DEFAULT_PORT)]
    port: usize,
    #[arg(long, default_value_t = String::from(consts::DEFAULT_IP))]
    ip: String,
    #[arg(long)]
    cfg: String,
    #[arg(long, default_value_t = false)]
    test_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cfg {
    rediscfg: String,
    dbcfg: String,
}

fn machine_id() -> u64 {
    static TMP: OnceLock<u64> = OnceLock::new();
    *TMP.get_or_init(|| {
        let state = path::Path::new("machine_id").exists();
        if state {
            return u64::from_be_bytes(fs::read("machine_id").unwrap().try_into().unwrap());
        }
        log::info!("Create machine id");
        let mut f = fs::File::create("machine_id").unwrap();
        let id: u64 = rand::thread_rng().gen_range(0..(1024 - 1));
        f.write_all(&id.to_be_bytes()).unwrap();
        id
    })
}

/// 真正被调用的主函数
pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();
    if parser.test_mode {
        let mut builder = env_logger::Builder::from_default_env();
        builder.target(env_logger::Target::Pipe(Box::new(
            fs::File::create("test.log").unwrap(),
        )));
        builder.init();
    } else {
        env_logger::init();
    }
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
    let mut server = server::Server::new(ip, port, db, redis, parser.test_mode).await?;
    tokio::spawn(async move {
        server
            .accept_sockets(shutdown_sender_clone, shutdown_receiver_clone)
            .await
    });
    let shutdown_sender_clone = shutdown_sender.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                log::info!("Exit because of ctrl-c signal");
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
        let mut shutdown_receiver = shutdown_sender.subscribe();
        loop {
            print!(">>>");
            std::io::stdout().flush().unwrap();
            let command = match console_reader.next_line().await {
                Ok(d) => match d {
                    Some(data) => data,
                    None => {
                        log::info!("Without stdin");
                        shutdown_receiver.recv().await.unwrap();
                        String::default()
                    }
                },
                Err(e) => {
                    log::error!("stdin {}", e);
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
            log::info!("Exit because command loop has exited");
        },
        _ = shutdown_receiver.recv() => {
            log::info!("Command loop exited");
        }
    }
    log::info!("Server exited");

    Ok(())
}
