#![feature(asm_goto)]
#![feature(decl_macro)]
#![feature(duration_constructors)]

mod cmd;
pub mod connection;
pub mod consts;
pub mod db;
mod entities;
pub mod requests;
mod server;
mod share_state;
pub mod utils;

use anyhow::bail;
use clap::Parser;
use cmd::CommandTransmitData;
use consts::{FileSize, DEFAULT_HTTP_PORT, DEFAULT_PORT};
use db::{file_storage, DbType};
use rand::Rng;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use server::httpserver;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};
use tokio::{
    net::TcpListener,
    select,
    sync::{broadcast, mpsc},
};
use tracing::instrument;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

type ShutdownRev = broadcast::Receiver<()>;
type ShutdownSdr = broadcast::Sender<()>;

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

#[derive(Debug, Serialize, Deserialize)]
struct Cfg {
    rediscfg: PathBuf,
    dbcfg: PathBuf,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    http_port: Option<u16>,
    #[serde(default)]
    db_type: DbType,
    #[serde(default = "consts::default_clear_interval")]
    auto_clean_duration: u64,
    #[serde(default = "consts::default_file_save_days")]
    file_save_days: u64,
    #[serde(default = "consts::default_enable_cmd")]
    enable_cmd: bool,
    #[serde(default = "consts::default_enable_cmd_stdin")]
    enable_cmd_stdin: bool,
    #[serde(default)]
    cmd_network_port: Option<u16>,
    #[serde(default = "consts::default_user_files_store_limit")]
    user_files_limit: FileSize,
    #[serde(default = "consts::default_friends_number_limit")]
    friends_number_limit: u32,
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
    let state = Path::new("machine_id").exists();
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
        let file = fs::File::create("test.log").unwrap();
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
    let dirpath = Path::new("log");
    if !dirpath.exists() {
        tracing::warn!("try clear log but not found");
        return Ok(());
    }
    fs::remove_dir_all(dirpath)?;
    fs::create_dir(dirpath)?;
    Ok(())
}

async fn cmd_start(
    command_rev: mpsc::Receiver<CommandTransmitData>,
    shutdown_sender: ShutdownSdr,
    db_conn: DatabaseConnection,
    test_mode: bool,
) -> anyhow::Result<()> {
    let mut shutdown_receiver1 = shutdown_sender.subscribe();
    if !test_mode {
        let mut shutdown_receiver2 = shutdown_sender.subscribe();
        select! {
            _ = cmd::cmd_process_loop(db_conn, command_rev) => {
                shutdown_sender.send(())?;
                tracing::info!("Exit because command loop has exited");
            },
            _ = shutdown_receiver2.recv() => {
                tracing::info!("Command loop exited");
            }
        }
    } else {
        shutdown_receiver1.recv().await?;
    }
    Ok(())
}

async fn exit_signal(shutdown_sender: ShutdownSdr) -> anyhow::Result<()> {
    let shutdown_sender_clone = shutdown_sender.clone();
    #[cfg(not(windows))]
    tokio::spawn(async move {
        if let Some(()) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?
            .recv()
            .await
        {
            tracing::info!("Exit because of sigterm signal");
            shutdown_sender.send(())?;
        }
        anyhow::Ok(())
    });
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
    Ok(())
}

async fn start_server(
    addr: (impl Into<String>, u16),
    db: DatabaseConnection,
    redis: redis::Client,
    test_mode: bool,
    http_sender: mpsc::Sender<httpserver::Record>,
    shutdown_sender: ShutdownSdr,
    shutdown_receiver: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let mut server = server::Server::new(addr, db, redis, http_sender, test_mode).await?;
    tokio::spawn(async move {
        server
            .accept_sockets(shutdown_sender, shutdown_receiver)
            .await
    });
    Ok(())
}

/// 启动网络cmd
async fn setup_network_cmd(
    cmd_port: u16,
    command_sdr: mpsc::Sender<CommandTransmitData>,
    mut shutdown_rev: ShutdownRev,
) -> anyhow::Result<()> {
    select! {
        err = cmd::setup_network(cmd_port, command_sdr) => {
            err?
        }
        _ = shutdown_rev.recv() => {}
    }
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
    let cfg_path = Path::new(&cfg_path).parent().unwrap();
    let mut cfg: Cfg = toml::from_str(&cfg)?;
    // 将相对路径转换
    cfg.convert_to_abs_path(cfg_path)?;
    // 配置端口
    let port = match parser.port {
        None => cfg.port.unwrap_or(DEFAULT_PORT),
        Some(port) => port,
    };
    let http_port = cfg.http_port.unwrap_or(DEFAULT_HTTP_PORT);
    let ip = parser.ip;
    // 启动维护模式
    unsafe {
        share_state::set_maintaining(parser.maintaining);
    }
    // 设置共享状态
    share_state::set_auto_clean_duration(cfg.auto_clean_duration);
    share_state::set_file_save_days(cfg.file_save_days);
    share_state::set_user_files_store_limit(cfg.user_files_limit);
    share_state::set_friends_number_limit(cfg.friends_number_limit);
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
    let (shutdown_sender, _) = broadcast::channel(32);
    // 构建http服务端
    let (handle, record_sender) = httpserver::HttpServer::new()
        .start(&ip, http_port, db.clone(), shutdown_sender.subscribe())
        .await?;
    // 构建websocket服务端
    start_server(
        (ip.clone(), port),
        db.clone(),
        redis,
        parser.test_mode,
        record_sender,
        shutdown_sender.clone(),
        shutdown_sender.subscribe(),
    )
    .await?;
    // 启动数据库文件系统
    file_storage::FileSys::new(db.clone()).start(shutdown_sender.subscribe());
    // 启动关闭信号监听
    exit_signal(shutdown_sender.clone()).await?;
    // 启动控制台
    if cfg.enable_cmd {
        let (command_sdr, command_rev) = mpsc::channel(50);
        match cfg.cmd_network_port {
            None => {
                // 不启动network cmd
            }
            Some(port) => {
                let command_sdr = command_sdr.clone();
                let shutdown_rev = shutdown_sender.subscribe();
                tokio::spawn(async move {
                    match setup_network_cmd(port, command_sdr, shutdown_rev).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("network cmd error:{}", e);
                        }
                    }
                });
            }
        }
        // 启动控制台源
        if cfg.enable_cmd_stdin {
            let shutdown_sender = shutdown_sender.clone();
            tokio::spawn(async move {
                match cmd::setup_stdin(shutdown_sender.subscribe(), command_sdr).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("cmd error:{}", e);
                    }
                }
            });
        }
        cmd_start(command_rev, shutdown_sender.clone(), db, parser.test_mode).await?;
    }
    handle.await??;
    tracing::info!("Server exited");
    Ok(())
}
