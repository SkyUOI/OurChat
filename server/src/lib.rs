pub mod connection;
pub mod consts;
mod db;
mod entities;
pub mod requests;
mod server;
pub mod utils;

use anyhow::bail;
use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path, sync::LazyLock};
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    select,
    sync::broadcast,
};
use tracing::instrument;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

type ShutdownRev = broadcast::Receiver<()>;

#[derive(Debug, Parser)]
#[command(author = "SkyUOI", version = base::build::VERSION, about = "The Server of OurChat")]
struct ArgsParser {
    #[arg(short, long, default_value_t = consts::DEFAULT_PORT)]
    port: usize,
    #[arg(long, default_value_t = String::from(consts::DEFAULT_IP))]
    ip: String,
    #[arg(long, default_value_t = String::default())]
    cfg: String,
    #[arg(long, default_value_t = false)]
    test_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cfg {
    rediscfg: String,
    dbcfg: String,
}

static MACHINE_ID: LazyLock<u64> = LazyLock::new(|| {
    let state = path::Path::new("machine_id").exists();
    if state {
        return u64::from_be_bytes(fs::read("machine_id").unwrap().try_into().unwrap());
    }
    tracing::info!("Create machine id");
    let mut f = fs::File::create("machine_id").unwrap();
    let id: u64 = rand::thread_rng().gen_range(0..(1024 - 1));
    f.write_all(&id.to_be_bytes()).unwrap();
    id
});

fn logger_init(test_mode: bool) -> WorkerGuard {
    let formatting_layer = fmt::layer()
        .pretty()
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::from_default_env());
    let (file_layer, guard) = if test_mode {
        // 是测试模式，记录到一个test.log中
        let file = std::fs::File::create("test.log").unwrap();
        let (non_blocking, guard) = tracing_appender::non_blocking(file);
        (
            fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking)
                .with_filter(EnvFilter::from_default_env()),
            guard,
        )
    } else {
        // 不是测试模式，记录按天滚动的日志
        let file_appender = tracing_appender::rolling::daily("log/", "ourchat");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        (
            fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking)
                .with_filter(EnvFilter::from_default_env()),
            guard,
        )
    };
    Registry::default()
        .with(formatting_layer)
        .with(file_layer)
        .init();
    guard
}

/// 真正被调用的主函数
#[instrument]
pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();

    let _forever = logger_init(parser.test_mode);
    let cfg_path = if parser.cfg.is_empty() {
        if let Ok(env) = std::env::var("OURCHAT_CONFIG_FILE") {
            env
        } else {
            tracing::error!("Please specify config file");
            bail!("Please specify config file");
        }
    } else {
        parser.cfg
    };
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
                tracing::info!("Exit because of ctrl-c signal");
                shutdown_sender_clone.send(())?;
            }
            Err(err) => {
                tracing::error!("Unable to listen to ctrl-c signal:{}", err);
                shutdown_sender_clone.send(())?;
            }
        }
        anyhow::Ok(())
    });
    if !parser.test_mode {
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
                            tracing::info!("Without stdin");
                            shutdown_receiver.recv().await.unwrap();
                            String::default()
                        }
                    },
                    Err(e) => {
                        tracing::error!("stdin {}", e);
                        break;
                    }
                };
                let command = command.trim();
                if command == "exit" {
                    tracing::info!("Exiting now...");
                    break;
                }
            }
            anyhow::Ok(())
        };
        select! {
            _ = input_loop => {
                shutdown_sender.send(())?;
                tracing::info!("Exit because command loop has exited");
            },
            _ = shutdown_receiver.recv() => {
                tracing::info!("Command loop exited");
            }
        }
    } else {
        shutdown_receiver.recv().await?;
    }
    tracing::info!("Server exited");
    Ok(())
}
