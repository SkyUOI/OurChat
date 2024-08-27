#![feature(asm_goto)]
#![feature(decl_macro)]

mod cmd;
pub mod connection;
pub mod consts;
pub mod db;
mod entities;
pub mod requests;
mod server;
pub mod utils;

use actix_web::{App, HttpServer};
use anyhow::bail;
use clap::Parser;
use consts::DEFAULT_PORT;
use db::DbType;
use rand::Rng;
use serde::{Deserialize, Serialize};
use static_keys::define_static_key_false;
use std::{
    fs,
    io::Write,
    path::{self, Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};
use tokio::{select, signal::unix::SignalKind, sync::broadcast};
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
    #[arg(short, long, help = "binding port")]
    port: Option<u16>,
    #[arg(long, default_value_t = String::from(consts::DEFAULT_IP), help = "binding ip")]
    ip: String,
    #[arg(long, default_value_t = String::default(), help = "ourchat config file path")]
    cfg: String,
    #[arg(
        long,
        default_value_t = false,
        help = "enable test mode(only for development)"
    )]
    test_mode: bool,
    #[arg(long, default_value_t = false, help = "clear files, such as logs")]
    clear: bool,
    #[arg(long, help = "database type,mysql or sqlite")]
    db_type: Option<String>,
    #[arg(
        long,
        help = "enable when server is maintaining",
        default_value_t = false
    )]
    maintaining: bool,
}

define_static_key_false!(MAINTAINING);

#[derive(Debug, Serialize, Deserialize)]
struct Cfg {
    rediscfg: PathBuf,
    dbcfg: PathBuf,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    db_type: DbType,
}

impl Cfg {
    fn convert_to_abs_path(&mut self, basepath: &Path) -> anyhow::Result<()> {
        let full_basepath = basepath.canonicalize()?;
        self.rediscfg = base::resolve_relative_path(&full_basepath, Path::new(&self.rediscfg))?;
        self.dbcfg = base::resolve_relative_path(&full_basepath, Path::new(&self.dbcfg))?;
        Ok(())
    }
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

fn clear() -> anyhow::Result<()> {
    let dirpath = std::path::Path::new("log");
    if !dirpath.exists() {
        tracing::warn!("try clear log but not found");
        return Ok(());
    }
    fs::remove_dir_all(dirpath)?;
    fs::create_dir(dirpath)?;
    Ok(())
}

/// 真正被调用的主函数
#[instrument]
pub async fn lib_main() -> anyhow::Result<()> {
    let parser = ArgsParser::parse();

    let _forever = logger_init(parser.test_mode);
    if parser.clear {
        clear()?;
    }
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
    // 读取配置文件
    let cfg = fs::read_to_string(&cfg_path)?;
    let cfg_path = path::Path::new(&cfg_path).parent().unwrap();
    let mut cfg: Cfg = toml::from_str(&cfg).unwrap();
    // 将相对路径转换
    cfg.convert_to_abs_path(cfg_path)?;
    // 配置端口
    let port = match parser.port {
        None => match cfg.port {
            Some(port) => port,
            None => DEFAULT_PORT,
        },
        Some(port) => port,
    };
    let ip = parser.ip;
    // 启动维护模式
    if parser.maintaining {
        unsafe {
            MAINTAINING.enable();
        }
        tracing::info!("Server is in maintaining mode");
    }
    // 处理数据库
    let db_type = match parser.db_type {
        None => cfg.db_type,
        Some(db_type) => match DbType::from_str(&db_type) {
            Ok(db_type) => db_type,
            Err(_) => bail!("Unknown database type. Only support mysql and sqlite"),
        },
    };
    db::init_db_system(db_type);
    let db = db::connect_to_db(&db::get_db_url(&cfg.dbcfg, cfg_path)?).await?;
    db::init_db(&db).await?;
    let redis = db::connect_to_redis(&db::get_redis_url(&cfg.rediscfg)?).await?;

    // 用于通知关闭的channel
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(32);
    let shutdown_sender_clone = shutdown_sender.clone();
    let shutdown_receiver_clone = shutdown_sender.subscribe();
    // 构建websocket服务端
    let mut server = server::Server::new(ip.clone(), port, db, redis, parser.test_mode).await?;
    tokio::spawn(async move {
        server
            .accept_sockets(shutdown_sender_clone, shutdown_receiver_clone)
            .await
    });
    // 构建http服务端
    let http_server = HttpServer::new(App::new).bind((ip.as_str(), 7778))?.run();
    let mut shutdown_receiver_http = shutdown_sender.subscribe();
    let http_server_handle = tokio::spawn(async move {
        select! {
            ret = http_server => {
                tracing::info!("Http server exited internally");
                ret
            }
            _ = shutdown_receiver_http.recv() => {
                tracing::info!("Http server exited by shutdown signal");
                Ok(())
            }
        }
    });

    let shutdown_sender_clone = shutdown_sender.clone();
    let shutdown_sender_clone2 = shutdown_sender.clone();
    #[cfg(not(windows))]
    tokio::spawn(async move {
        if let Some(()) = tokio::signal::unix::signal(SignalKind::terminate())?
            .recv()
            .await
        {
            tracing::info!("Exit because of ctrl-c signal");
            shutdown_sender_clone.send(())?;
        }
        anyhow::Ok(())
    });
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("Exit because of ctrl-c signal");
                shutdown_sender_clone2.send(())?;
            }
            Err(err) => {
                tracing::error!("Unable to listen to ctrl-c signal:{}", err);
                shutdown_sender_clone2.send(())?;
            }
        }
        anyhow::Ok(())
    });

    if !parser.test_mode {
        let mut shutdown_receiver2 = shutdown_sender.subscribe();
        select! {
            _ = cmd::cmd_process_loop(shutdown_receiver) => {
                shutdown_sender.send(())?;
                tracing::info!("Exit because command loop has exited");
            },
            _ = shutdown_receiver2.recv() => {
                tracing::info!("Command loop exited");
            }
        }
    } else {
        shutdown_receiver.recv().await?;
    }
    http_server_handle.await??;
    tracing::info!("Server exited");
    Ok(())
}
