#![feature(decl_macro)]
#![feature(duration_constructors)]

pub mod basic;
mod cmd;
pub mod component;
pub mod consts;
pub mod db;
mod entities;
pub mod pb;
pub mod process;
mod server;
mod shared_state;
pub mod utils;

use crate::component::EmailClient;
use anyhow::bail;
pub use basic::*;
use clap::Parser;
use cmd::CommandTransmitData;
use component::EmailSender;
use config::File;
use consts::{CONFIG_FILE_ENV_VAR, FileSize, ID, LOG_ENV_VAR, LOG_OUTPUT_DIR, STDIN_AVAILABLE};
use dashmap::DashMap;
use db::{DbCfgTrait, PostgresDbCfg, file_storage};
use futures_util::future::join_all;
use lettre::{AsyncSmtpTransport, transport::smtp::authentication::Credentials};
use parking_lot::Once;
use pb::msg_delivery::Msg;
use rand::Rng;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use server::httpserver;
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock},
};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug, Parser, Default)]
#[command(author = "SkyUOI", version = base::build::VERSION, about = "The Server of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "binding port")]
    pub port: Option<u16>,
    #[arg(long, help = "http server binding port")]
    pub http_port: Option<u16>,
    #[arg(long, help = "binding ip")]
    pub ip: Option<String>,
    #[command(flatten)]
    pub shared_cfg: ParserCfg,
    #[arg(long, help = "whether to enable cmd")]
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
    #[serde(default = "consts::default_verification_expire_days")]
    pub verification_expire_days: u64,
    // #[serde(default = "consts::default_password_hash_algorithm"))]
    // pub password_hash_algorithm: argon2::Algorithm,
    pub db: OCDbCfg,

    #[serde(skip)]
    pub cmd_args: ParserCfg,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OCDbCfg {
    #[serde(default = "consts::default_fetch_msg_page_size")]
    pub fetch_msg_page_size: u64,
}

impl MainCfg {
    pub fn email_available(&self) -> bool {
        self.email_address.is_some() && self.smtp_address.is_some() && self.smtp_password.is_some()
    }

    pub fn build_email_client(&self) -> anyhow::Result<EmailClient> {
        if !self.email_available() {
            bail!("email is not available");
        }
        let creds = Credentials::new(
            self.email_address.clone().unwrap(),
            self.smtp_password.clone().unwrap(),
        );
        EmailClient::new(
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(
                &self.smtp_address.clone().unwrap(),
            )?
            .credentials(creds)
            .build(),
            self.email_address.as_ref().unwrap(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Cfg {
    pub main_cfg: MainCfg,
    pub db_cfg: PostgresDbCfg,
}

impl Cfg {
    pub fn new(main_cfg: MainCfg) -> anyhow::Result<Self> {
        let dbcfg = {
            let cfg = config::Config::builder()
                .add_source(config::File::with_name(main_cfg.dbcfg.to_str().unwrap()))
                .build()?;
            let cfg: PostgresDbCfg = cfg.try_deserialize()?;
            cfg
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

static SERVER_INFO_PATH: &str = "server_info.json";

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    unique_id: uuid::Uuid,
    machine_id: u64,
    secret: String,
}

const SECRET_LEN: usize = 32;

static SERVER_INFO: LazyLock<ServerInfo> = LazyLock::new(|| {
    let state = Path::new(SERVER_INFO_PATH).exists();
    if state {
        let info = match serde_json::from_str(&fs::read_to_string(SERVER_INFO_PATH).unwrap()) {
            Ok(info) => info,
            Err(e) => {
                tracing::error!(
                    "read server info error:{}.You can try modify the file \"{}\" to satisfy the requirement,or you can delete the file and rerun the server to generate a new file",
                    e,
                    SERVER_INFO_PATH
                );
                std::process::exit(1);
            }
        };
        return info;
    }
    tracing::info!("Create server info file");

    let mut f = fs::File::create(SERVER_INFO_PATH).unwrap();
    let id: u64 = rand::thread_rng().gen_range(0..(1024 - 1));
    let info = ServerInfo {
        unique_id: uuid::Uuid::new_v4(),
        machine_id: id,
        secret: utils::generate_random_string(SECRET_LEN),
    };
    serde_json::to_writer(&mut f, &info).unwrap();
    info
});

/// # Warning
/// This function should be called only once.The second one will be ignored
fn logger_init<Sink>(test_mode: bool, source: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    static INIT: OnceLock<Option<WorkerGuard>> = OnceLock::new();
    INIT.get_or_init(|| {
        let env = if test_mode {
            || EnvFilter::try_from_env(LOG_ENV_VAR).unwrap_or("trace".into())
        } else {
            || EnvFilter::try_from_env(LOG_ENV_VAR).unwrap_or("info".into())
        };
        let formatting_layer = fmt::layer().pretty().with_writer(source);
        let file_appender = if test_mode {
            tracing_appender::rolling::never(LOG_OUTPUT_DIR, "test")
        } else {
            tracing_appender::rolling::daily(LOG_OUTPUT_DIR, "ourchat")
        };
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);
        Registry::default()
            .with(env())
            .with(formatting_layer)
            .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
            .init();
        Some(file_guard)
    });
}

fn clear() -> anyhow::Result<()> {
    let dirpath = Path::new(LOG_OUTPUT_DIR);
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
    if !test_mode {
        match cmd::cmd_process_loop(db_conn, command_rev, shutdown_sender.clone()).await {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("cmd error:{}", e);
            }
        };
    } else {
        let mut shutdown_receiver = shutdown_sender.new_receiver("cmd process loop", "cmd loop");
        shutdown_receiver.wait_shutdowning().await;
    }
    Ok(())
}

fn exit_signal(mut shutdown_sender: ShutdownSdr) -> anyhow::Result<()> {
    let mut shutdown_sender_clone = shutdown_sender.clone();
    #[cfg(not(windows))]
    tokio::spawn(async move {
        if let Some(()) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?
            .recv()
            .await
        {
            tracing::info!("Exit because of sigterm signal");
            shutdown_sender.shutdown_all_tasks().await?;
        }
        anyhow::Ok(())
    });
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("Exit because of ctrl-c signal");
                shutdown_sender_clone.shutdown_all_tasks().await?;
            }
            Err(err) => {
                tracing::error!("Unable to listen to ctrl-c signal:{}", err);
                shutdown_sender_clone.shutdown_all_tasks().await?;
            }
        }
        anyhow::Ok(())
    });
    Ok(())
}

#[derive(Clone)]
pub struct HttpSender {}

/// build websocket server
async fn start_server(
    addr: impl Into<SocketAddr>,
    db: DbPool,
    http_sender: HttpSender,
    shared_data: Arc<SharedData<impl EmailSender>>,
    shutdown_receiver: ShutdownRev,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let server = server::RpcServer::new(addr, db, http_sender, shared_data);
    let handle = tokio::spawn(async move { server.run(shutdown_receiver).await });
    Ok(handle)
}

/// global init,can be called many times,but only the first time will be effective
fn global_init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {})
}

pub struct Application<T: EmailSender> {
    pub shared: Arc<SharedData<T>>,
    pub pool: DbPool,
    server_addr: SocketAddr,
    http_listener: Option<std::net::TcpListener>,
    /// for shutdowning server fully,you shouldn't use handle.abort() to do this
    abort_sender: ShutdownSdr,
    pub started_notify: Arc<tokio::sync::Notify>,
}

#[derive(Debug, Clone)]
/// The database connection pool, redis connection pool
/// you can clone it freely without many extra cost
pub struct DbPool {
    pub db_pool: DatabaseConnection,
    pub redis_pool: deadpool_redis::Pool,
}

impl DbPool {
    async fn close(&mut self) -> anyhow::Result<()> {
        self.db_pool.clone().close().await?;
        self.redis_pool.close();
        Ok(())
    }
}

/// shared data along the whole application
pub struct SharedData<T: EmailSender> {
    pub email_client: Option<T>,
    pub cfg: Cfg,
    pub verify_record: DashMap<String, Arc<tokio::sync::Notify>>,
    pub connected_clients: DashMap<ID, mpsc::Sender<Msg>>,
}

pub fn get_configuration(config_path: Option<impl Into<PathBuf>>) -> anyhow::Result<MainCfg> {
    let cfg_path = match config_path {
        Some(cfg_path) => cfg_path.into(),
        None => if let Ok(env) = std::env::var(CONFIG_FILE_ENV_VAR) {
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

impl<T: EmailSender> Application<T> {
    pub async fn build(
        parser: ArgsParser,
        mut cfg: Cfg,
        email_client: Option<T>,
    ) -> anyhow::Result<Self> {
        global_init();
        let main_cfg = &mut cfg.main_cfg;

        if main_cfg.cmd_args.test_mode {
            logger_init(main_cfg.cmd_args.test_mode, std::io::sink);
        } else {
            logger_init(main_cfg.cmd_args.test_mode, std::io::stdout);
        }
        // start maintain mode
        shared_state::set_maintaining(main_cfg.cmd_args.maintaining);
        // Set up shared state
        shared_state::set_auto_clean_duration(main_cfg.auto_clean_duration);
        shared_state::set_file_save_days(main_cfg.file_save_days);
        shared_state::set_user_files_store_limit(main_cfg.user_files_limit);
        shared_state::set_friends_number_limit(main_cfg.friends_number_limit);

        if let Some(new_ip) = parser.ip {
            main_cfg.ip = new_ip;
        }

        // port
        let port = match parser.port {
            None => main_cfg.port,
            Some(port) => port,
        };
        let addr: SocketAddr = format!("{}:{}", &main_cfg.ip, port).parse().unwrap();
        main_cfg.port = addr.port();

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
        main_cfg.http_port = http_listener.local_addr()?.port();

        // enable cmd
        let enable_cmd = match parser.enable_cmd {
            None => main_cfg.enable_cmd,
            Some(enable_cmd) => enable_cmd,
        };
        main_cfg.enable_cmd = enable_cmd;
        let abort_sender = ShutdownSdr::new(None);
        // connect to database
        db::init_db_system();
        let db = db::connect_to_db(&cfg.db_cfg.url()).await?;
        db::init_db(&db).await?;
        // connect to redis
        let redis = db::connect_to_redis(&db::get_redis_url(&main_cfg.rediscfg)?).await?;

        Ok(Self {
            shared: Arc::new(SharedData {
                email_client,
                cfg,
                verify_record: DashMap::new(),
                connected_clients: DashMap::new(),
            }),
            pool: DbPool {
                redis_pool: redis,
                db_pool: db,
            },
            server_addr: addr,
            http_listener: Some(http_listener),
            abort_sender,
            started_notify: Arc::new(tokio::sync::Notify::new()),
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
        tracing::info!("Starting server");
        let cfg = &self.shared.cfg.main_cfg;

        if cfg.cmd_args.clear {
            clear()?;
        }

        let mut handles = Vec::new();
        // Build http server
        let (handle, record_sender) = httpserver::HttpServer::new()
            .start(
                self.http_listener.take().unwrap(),
                self.pool.clone(),
                self.shared.clone(),
                self.abort_sender.clone(),
            )
            .await?;
        handles.push(handle);

        let handle = start_server(
            self.server_addr,
            self.pool.clone(),
            record_sender,
            self.shared.clone(),
            self.abort_sender.new_receiver("rpc server", "rpc server"),
        )
        .await?;
        handles.push(handle);

        // Start the database file system
        file_storage::FileSys::new(self.pool.db_pool.clone())
            .start(self.abort_sender.new_receiver("file system", "file system"));
        // Start the shutdown signal listener
        exit_signal(self.abort_sender.clone())?;
        // Start the cmd
        if cfg.enable_cmd {
            let (command_sdr, command_rev) = mpsc::channel(50);
            match cfg.cmd_network_port {
                None => {
                    // not start network cmd
                }
                Some(port) => {
                    let command_sdr = command_sdr.clone();
                    let shutdown_rev = self
                        .abort_sender
                        .new_receiver("network cmd", "network source");
                    tokio::spawn(async move {
                        match cmd::setup_network(port, command_sdr, shutdown_rev).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("network cmd error:{}", e);
                            }
                        }
                    });
                }
            }
            // Start the cmd from stdin
            if cfg.enable_cmd_stdin && *STDIN_AVAILABLE {
                let shutdown_sender = self.abort_sender.clone();
                tokio::spawn(async move {
                    match cmd::setup_stdin(
                        command_sdr,
                        shutdown_sender.new_receiver("stdin cmd", "stdin source"),
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("cmd error:{}", e);
                        }
                    }
                });
            }
            tokio::spawn(cmd_start(
                command_rev,
                self.abort_sender.clone(),
                self.pool.db_pool.clone(),
                cfg.cmd_args.test_mode,
            ));
        }
        tracing::info!("Server started");
        self.started_notify.notify_waiters();
        join_all(handles).await.iter().for_each(|x| {
            if let Err(e) = x {
                tracing::error!("server error:{}", e);
            }
        });
        self.pool.close().await?;
        tracing::info!("Server exited");
        Ok(())
    }
}
