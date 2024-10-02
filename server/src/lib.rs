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
mod shared_state;
pub mod utils;

use anyhow::bail;
use clap::Parser;
use cmd::CommandTransmitData;
use config::File;
use consts::{FileSize, STDIN_AVAILABLE};
use db::{file_storage, DbCfg, DbType, MysqlDbCfg, SqliteDbCfg};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport};
use parking_lot::Once;
use rand::Rng;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use server::httpserver;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, LazyLock, OnceLock},
};
use tokio::{
    net::TcpListener,
    select,
    sync::{broadcast, mpsc, oneshot},
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub type ShutdownRev = broadcast::Receiver<()>;
pub type ShutdownSdr = broadcast::Sender<()>;

#[derive(Debug, Parser, Default)]
#[command(author = "SkyUOI", version = base::build::VERSION, about = "The Server of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "binding port")]
    pub port: Option<u16>,
    #[arg(long, help = "http server binding port")]
    pub http_port: Option<u16>,
    #[arg(long, help = "binding ip")]
    pub ip: Option<String>,
    #[arg(long, help = "database type,mysql or sqlite")]
    pub db_type: Option<String>,
    #[command(flatten)]
    pub shared_cfg: ParserCfg,
    #[arg(long, help = "if enable cmd")]
    pub enable_cmd: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainCfg {
    #[serde(default = "consts::default_ip")]
    pub ip: String,
    pub rediscfg: PathBuf,
    pub dbcfg: PathBuf,
    #[serde(default = "consts::default_port")]
    pub port: u16,
    #[serde(default = "consts::default_http_port")]
    pub http_port: u16,
    #[serde(default)]
    pub db_type: DbType,
    #[serde(default = "consts::default_clear_interval")]
    pub auto_clean_duration: u64,
    #[serde(default = "consts::default_file_save_days")]
    pub file_save_days: u64,
    #[serde(default = "consts::default_enable_cmd")]
    pub enable_cmd: bool,
    #[serde(default = "consts::default_enable_cmd_stdin")]
    pub enable_cmd_stdin: bool,
    #[serde(default)]
    pub cmd_network_port: Option<u16>,
    #[serde(default = "consts::default_user_files_store_limit")]
    pub user_files_limit: FileSize,
    #[serde(default = "consts::default_friends_number_limit")]
    pub friends_number_limit: u32,
    #[serde(default)]
    pub email_address: Option<String>,
    #[serde(default)]
    pub smtp_address: Option<String>,
    #[serde(default)]
    pub smtp_password: Option<String>,
    #[serde(skip)]
    pub cmd_args: ParserCfg,
}

#[derive(Debug, Clone)]
pub struct Cfg {
    pub main_cfg: MainCfg,
    pub db_cfg: DbCfg,
}

impl Cfg {
    pub fn new(main_cfg: MainCfg) -> anyhow::Result<Self> {
        let dbcfg = match main_cfg.db_type {
            DbType::Sqlite => {
                let cfg = config::Config::builder()
                    .add_source(config::File::with_name(main_cfg.dbcfg.to_str().unwrap()))
                    .build()?;
                let mut cfg: SqliteDbCfg = cfg.try_deserialize()?;
                cfg.convert_to_abs_path(
                    main_cfg.cmd_args.config.as_ref().unwrap().parent().unwrap(),
                )?;
                DbCfg::Sqlite(cfg)
            }
            DbType::MySql => {
                let cfg = config::Config::builder()
                    .add_source(config::File::with_name(main_cfg.dbcfg.to_str().unwrap()))
                    .build()?;
                let cfg: MysqlDbCfg = cfg.try_deserialize()?;
                DbCfg::Mysql(cfg)
            }
        };
        Ok(Self {
            main_cfg,
            db_cfg: dbcfg,
        })
    }
}

#[derive(Debug, Parser, Clone, Default)]
pub struct ParserCfg {
    #[arg(
        long,
        default_value_t = false,
        help = "enable test mode(only for development)"
    )]
    pub test_mode: bool,
    #[arg(long, default_value_t = false, help = "clear files, such as logs")]
    pub clear: bool,
    #[arg(
        long,
        help = "enable when server is maintaining",
        default_value_t = false
    )]
    pub maintaining: bool,
    #[arg(long, help = "ourchat config file path")]
    pub config: Option<PathBuf>,
}

impl MainCfg {
    fn convert_to_abs_path(&mut self) -> anyhow::Result<()> {
        let full_basepath = self
            .cmd_args
            .config
            .as_ref()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()?;
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

/// # Warning
/// This function should be called only once.The second one will be ignored
fn logger_init(test_mode: bool) {
    static INIT: OnceLock<WorkerGuard> = OnceLock::new();
    INIT.get_or_init(|| {
        let env = if test_mode {
            || EnvFilter::try_from_default_env().unwrap_or("trace".into())
        } else {
            || EnvFilter::try_from_default_env().unwrap_or("info".into())
        };
        let formatting_layer = fmt::layer()
            .pretty()
            .with_writer(std::io::stdout)
            .with_filter(env());
        let registry = Registry::default().with(formatting_layer);

        let file_appender = tracing_appender::rolling::daily("log/", "ourchat");
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);
        registry
            .with(
                fmt::layer()
                    .with_ansi(false)
                    .with_writer(non_blocking)
                    .with_filter(env()),
            )
            .init();
        file_guard
    });
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
            _ = cmd::cmd_process_loop(db_conn, command_rev, shutdown_receiver1) => {
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

#[derive(Clone)]
pub struct HttpSender {
    file_record: mpsc::Sender<httpserver::FileRecord>,
    verify_record: mpsc::Sender<(
        httpserver::VerifyRecord,
        oneshot::Sender<anyhow::Result<()>>,
    )>,
}

/// build websocket server
async fn start_server(
    listener: TcpListener,
    db: DatabaseConnection,
    redis: redis::Client,
    http_sender: HttpSender,
    shared_data: Arc<SharedData>,
    shutdown_sender: ShutdownSdr,
    shutdown_receiver: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let mut server = server::Server::new(listener, db, redis, http_sender, shared_data).await?;
    tokio::spawn(async move {
        server
            .accept_sockets(shutdown_sender, shutdown_receiver)
            .await
    });
    Ok(())
}

fn global_init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        static_keys::global_init();
    })
}

pub struct Application {
    shared: Arc<SharedData>,
    server_listener: Option<TcpListener>,
    http_listener: Option<std::net::TcpListener>,
    /// for shutdowning server fully,you shouldn't use handle.abort() to do this
    abort_sender: ShutdownSdr,
}

struct SharedState {
    email_available: bool,
    email_client: Option<EmailClient>,
}

pub struct SharedData {
    shared_state: SharedState,
    cfg: Cfg,
}

type EmailClient = AsyncSmtpTransport<lettre::Tokio1Executor>;

fn build_email_client(cfg: &MainCfg) -> EmailClient {
    let creds = Credentials::new(
        cfg.email_address.clone().unwrap(),
        cfg.smtp_password.clone().unwrap(),
    );
    AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&cfg.smtp_address.clone().unwrap())
        .unwrap()
        .credentials(creds)
        .build()
}

pub fn get_configuration(config_path: Option<impl Into<PathBuf>>) -> anyhow::Result<MainCfg> {
    let cfg_path = match config_path {
        Some(cfg_path) => cfg_path.into(),
        None => if let Ok(env) = std::env::var("OURCHAT_CONFIG_FILE") {
            env
        } else {
            tracing::error!("Please specify config file");
            bail!("Please specify config file");
        }
        .into(),
    };
    // read config file
    let cfg = config::Config::builder()
        .add_source(File::with_name(cfg_path.to_str().unwrap()))
        .build()
        .expect("Failed to build config");
    let mut cfg: MainCfg = cfg.try_deserialize().expect("wrong config file");
    cfg.cmd_args.config = Some(cfg_path);
    // convert the path relevant to the config file to path relevant to the directory
    cfg.convert_to_abs_path()?;
    Ok(cfg)
}

impl Application {
    pub async fn build(parser: ArgsParser, mut cfg: Cfg) -> anyhow::Result<Self> {
        let main_cfg = &mut cfg.main_cfg;
        if let Some(new_ip) = parser.ip {
            main_cfg.ip = new_ip;
        }

        // port
        let port = match parser.port {
            None => main_cfg.port,
            Some(port) => port,
        };
        let addr = format!("{}:{}", &main_cfg.ip, port);
        let server_listener = TcpListener::bind(&addr).await?;
        main_cfg.port = server_listener.local_addr().unwrap().port();

        // http port
        let http_port = match parser.http_port {
            None => main_cfg.http_port,
            Some(http_port) => http_port,
        };
        let ip = main_cfg.ip.clone();
        let http_listener = tokio::task::spawn_blocking(move || {
            std::net::TcpListener::bind(format!("{}:{}", &ip, http_port))
        })
        .await??;
        main_cfg.http_port = http_listener.local_addr().unwrap().port();

        // database type
        let db_type = match parser.db_type {
            None => main_cfg.db_type,
            Some(db_type) => match DbType::from_str(&db_type) {
                Ok(db_type) => db_type,
                Err(_) => bail!("Unknown database type. Only support mysql and sqlite"),
            },
        };
        main_cfg.db_type = db_type;
        let email_available = main_cfg.email_address.is_some()
            && main_cfg.smtp_password.is_some()
            && main_cfg.smtp_address.is_some();
        let email_client = if email_available {
            Some(build_email_client(main_cfg))
        } else {
            None
        };

        // enable cmd
        let enable_cmd = match parser.enable_cmd {
            None => main_cfg.enable_cmd,
            Some(enable_cmd) => enable_cmd,
        };
        main_cfg.enable_cmd = enable_cmd;
        let (abort_sender, _) = broadcast::channel(20);

        Ok(Self {
            shared: Arc::new(SharedData {
                shared_state: SharedState {
                    email_available,
                    email_client,
                },
                cfg,
            }),
            server_listener: Some(server_listener),
            http_listener: Some(http_listener),
            abort_sender,
        })
    }

    pub fn get_port(&self) -> u16 {
        self.shared.cfg.main_cfg.port
    }

    pub fn get_http_port(&self) -> u16 {
        self.shared.cfg.main_cfg.http_port
    }

    pub fn get_abort_handle(&self) -> ShutdownSdr {
        self.abort_sender.clone()
    }

    pub async fn run_forever(&mut self) -> anyhow::Result<()> {
        let cfg = &self.shared.cfg.main_cfg;
        global_init();

        logger_init(cfg.cmd_args.test_mode);
        if cfg.cmd_args.clear {
            clear()?;
        }

        // start maintain mode
        shared_state::set_maintaining(cfg.cmd_args.maintaining);
        // Set up shared state
        shared_state::set_auto_clean_duration(cfg.auto_clean_duration);
        shared_state::set_file_save_days(cfg.file_save_days);
        shared_state::set_user_files_store_limit(cfg.user_files_limit);
        shared_state::set_friends_number_limit(cfg.friends_number_limit);

        db::init_db_system(cfg.db_type);
        let db = db::connect_to_db(&db::get_db_url(&self.shared.cfg.db_cfg)?).await?;
        db::init_db(&db).await?;
        let redis = db::connect_to_redis(&db::get_redis_url(&cfg.rediscfg)?).await?;

        // Build http server
        let (handle, record_sender) = httpserver::HttpServer::new()
            .start(
                self.http_listener.take().unwrap(),
                db.clone(),
                self.shared.clone(),
                self.abort_sender.subscribe(),
            )
            .await?;

        start_server(
            self.server_listener.take().unwrap(),
            db.clone(),
            redis,
            record_sender,
            self.shared.clone(),
            self.abort_sender.clone(),
            self.abort_sender.subscribe(),
        )
        .await?;
        // 启动数据库文件系统
        file_storage::FileSys::new(db.clone()).start(self.abort_sender.subscribe());
        // 启动关闭信号监听
        exit_signal(self.abort_sender.clone()).await?;
        // 启动控制台
        if cfg.enable_cmd {
            let mut is_enable = false;
            let (command_sdr, command_rev) = mpsc::channel(50);
            match cfg.cmd_network_port {
                None => {
                    // 不启动network cmd
                }
                Some(port) => {
                    let command_sdr = command_sdr.clone();
                    let shutdown_rev = self.abort_sender.subscribe();
                    tokio::spawn(async move {
                        match cmd::setup_network(port, command_sdr, shutdown_rev).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("network cmd error:{}", e);
                            }
                        }
                    });
                    is_enable = true;
                }
            }
            // 启动控制台源
            if cfg.enable_cmd_stdin && *STDIN_AVAILABLE {
                let shutdown_sender = self.abort_sender.clone();
                tokio::spawn(async move {
                    match cmd::setup_stdin(command_sdr, shutdown_sender.subscribe()).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("cmd error:{}", e);
                        }
                    }
                });
                is_enable = true;
            }
            cmd_start(
                command_rev,
                self.abort_sender.clone(),
                db.clone(),
                cfg.cmd_args.test_mode,
            )
            .await?;
            if !is_enable {
                // 控制台并未正常启动
                tracing::warn!("cmd is not enabled");
                self.abort_sender.subscribe().recv().await?;
            }
        }
        handle.await??;
        db.close().await?;
        tracing::info!("Server exited");
        Ok(())
    }
}
