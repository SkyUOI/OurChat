#![feature(decl_macro)]
#![feature(duration_constructors)]

pub mod basic;
mod cmd;
pub mod component;
pub mod consts;
mod cryption;
pub mod db;
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
use config::{ConfigError, File};
use consts::{CONFIG_FILE_ENV_VAR, ID, LOG_ENV_VAR, LOG_OUTPUT_DIR, STDIN_AVAILABLE};
use dashmap::DashMap;
use db::{PostgresDbCfg, file_storage};
use futures_util::future::join_all;
use lettre::{AsyncSmtpTransport, transport::smtp::authentication::Credentials};
use parking_lot::Once;
use pb::ourchat::msg_delivery::v1::FetchMsgsResponse;
use rand::Rng;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use server::httpserver;
use size::Size;
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock},
    time::Duration,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

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
    pub user_files_limit: Size,
    #[serde(default = "consts::default_friends_number_limit")]
    pub friends_number_limit: u32,
    #[serde(default = "consts::default_files_storage_path")]
    pub files_storage_path: PathBuf,
    #[serde(default = "consts::default_verification_expire_days")]
    pub verification_expire_days: u64,
    #[serde(default = "consts::default_ssl")]
    pub ssl: bool,
    #[serde(default = "consts::default_single_instance")]
    pub single_instance: bool,
    pub password_hash: PasswordHash,
    pub db: OCDbCfg,
    pub debug: DebugCfg,
    pub email: EmailCfg,
    pub registry: RegistryCfg,

    #[serde(skip)]
    pub cmd_args: ParserCfg,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordHash {
    #[serde(default = "consts::default_m_cost")]
    pub m_cost: u32,
    #[serde(default = "consts::default_t_cost")]
    pub t_cost: u32,
    #[serde(default = "consts::default_p_cost")]
    pub p_cost: u32,
    #[serde(default = "consts::default_output_len")]
    pub output_len: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugCfg {
    #[serde(default = "consts::default_debug_console")]
    pub debug_console: bool,
    #[serde(default = "consts::default_debug_console_port")]
    pub debug_console_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryCfg {
    #[serde(default = "consts::default_enable_registry")]
    pub enable: bool,
    #[serde(default = "consts::default_registry_port")]
    pub port: u16,
    #[serde(default = "consts::default_registry_ip")]
    pub ip: String,
    #[serde(default = "consts::default_service_name")]
    pub service_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailCfg {
    #[serde(default = "consts::default_enable_email")]
    pub enable: bool,
    #[serde(default)]
    pub email_address: Option<String>,
    #[serde(default)]
    pub smtp_address: Option<String>,
    #[serde(default)]
    pub smtp_password: Option<String>,
}

impl EmailCfg {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OCDbCfg {
    #[serde(default = "consts::default_fetch_msg_page_size")]
    pub fetch_msg_page_size: u64,
}

fn read_a_config(path: impl AsRef<Path>) -> Result<config::Config, ConfigError> {
    config::Config::builder()
        .add_source(File::with_name(path.as_ref().to_str().unwrap()))
        .build()
}

impl MainCfg {
    pub fn new(config_path: Vec<impl Into<PathBuf>>) -> anyhow::Result<Self> {
        let len = config_path.len();
        let mut iter = config_path.into_iter();
        let cfg_path = if len == 0 {
            if let Ok(env) = std::env::var(CONFIG_FILE_ENV_VAR) {
                env
            } else {
                tracing::error!("Please specify config file");
                bail!("Please specify config file");
            }
            .into()
        } else {
            iter.next().unwrap().into()
        };
        // read config file
        let mut cfg: MainCfg = read_a_config(&cfg_path)
            .expect("Failed to build config")
            .try_deserialize()
            .expect("Wrong config file structure");
        let mut configs_list = vec![cfg_path];
        for i in iter {
            configs_list.push(i.into());
            // TODO: Merge
        }
        cfg.cmd_args.config = configs_list;
        // convert the path relevant to the config file to path relevant to the directory
        cfg.convert_to_abs_path()?;
        Ok(cfg)
    }

    pub fn protocol_http(&self) -> String {
        if self.ssl {
            "https".to_string()
        } else {
            "http".to_string()
        }
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
    #[arg(short, long, help = "ourchat config file path", num_args = 0..,)]
    pub config: Vec<PathBuf>,
}

impl MainCfg {
    fn convert_to_abs_path(&mut self) -> anyhow::Result<()> {
        let full_basepath = self
            .cmd_args
            .config
            .first()
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

/// Initialize the logger.
///
/// If `test_mode` is `true`, it will always set the log level to "trace".
/// Otherwise, it will read the log level from the environment variable
/// specified by `LOG_ENV_VAR` and set it to "info" if not present.
/// The log will be written to a file in the directory specified by
/// `LOG_OUTPUT_DIR` and the file name will be "test" if `test_mode` is
/// `true` and "ourchat" otherwise.
/// If `debug_cfg` is `Some` and `debug_console` is `true`, it will also
/// write the log to the console at the address specified by
/// `debug_cfg.debug_console_port`.
///
/// # Warning
/// This function should be called only once.
/// The second one will be ignored
pub fn logger_init<Sink>(test_mode: bool, debug_cfg: Option<&DebugCfg>, output: Sink)
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
        let formatting_layer = fmt::layer().pretty().with_writer(output);
        let file_appender = if test_mode {
            tracing_appender::rolling::never(LOG_OUTPUT_DIR, "test")
        } else {
            tracing_appender::rolling::daily(LOG_OUTPUT_DIR, "ourchat")
        };
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);
        let tmp = Registry::default()
            .with(env())
            .with(formatting_layer)
            .with(fmt::layer().with_ansi(false).with_writer(non_blocking));
        if let Some(debug_cfg) = debug_cfg {
            if debug_cfg.debug_console {
                // TODO:move this to "debug" section of config
                let console_layer = console_subscriber::ConsoleLayer::builder()
                    .retention(Duration::from_secs(60))
                    .server_addr(([0, 0, 0, 0], debug_cfg.debug_console_port))
                    .spawn();
                tmp.with(console_layer).init();
            }
        } else {
            tmp.init();
        }
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

fn exit_signal(#[allow(unused_mut)] mut shutdown_sender: ShutdownSdr) -> anyhow::Result<()> {
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

#[derive(Debug, Clone)]
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

/// The database connection pool, redis connection pool
/// you can clone it freely without many extra cost
#[derive(Debug, Clone)]
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
#[derive(Debug)]
pub struct SharedData<T: EmailSender> {
    pub email_client: Option<T>,
    pub cfg: Cfg,
    pub verify_record: DashMap<String, Arc<tokio::sync::Notify>>,
    pub connected_clients: DashMap<ID, mpsc::Sender<Result<FetchMsgsResponse, tonic::Status>>>,
}

/// Loads and constructs the configuration for the application.
///
/// This function takes a list of paths to configuration files, attempts to read
/// them, and deserializes the configuration data into the `Cfg` structure.
/// If no paths are provided, it attempts to read from the environment variable
/// specified by `CONFIG_FILE_ENV_VAR`.
///
/// # Arguments
///
/// * `config_path` - A vector of paths pointing to configuration files. Each path
///   is convertible into a `PathBuf`.
///
/// # Returns
///
/// Returns a `Result` containing the `Cfg` if successful, or an error if it fails
/// to read or parse the configuration files.
///
/// # Errors
///
/// This function will return an error if it is unable to read or deserialize any
/// of the specified configuration files or if the environment variable is not set
/// when no paths are provided.
pub fn get_configuration(config_path: Vec<impl Into<PathBuf>>) -> anyhow::Result<Cfg> {
    let main_cfg = MainCfg::new(config_path)?;
    Cfg::new(main_cfg)
}

pub async fn register_service(cfg: &RegistryCfg) -> anyhow::Result<()> {
    Ok(())
}

impl<T: EmailSender> Application<T> {
    /// Builds a new `Application` instance.
    ///
    /// This function will setup the log system, shared state, connect to the database,
    /// and connect to Redis. The `parser` argument is used to override some of the
    /// configuration if specified. The `cfg` argument is the configuration to be used.
    ///
    /// # Arguments
    ///
    /// * `parser` - The parsed command line arguments.
    /// * `cfg` - The configuration to be used.
    /// * `email_client` - The email client to be used. If `None`, the email client
    ///   will be ignored.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Application` instance if successful, or an error
    /// if it fails to set up the log system, shared state, connect to the database, or
    /// connect to Redis.
    pub async fn build(
        parser: ArgsParser,
        mut cfg: Cfg,
        email_client: Option<T>,
    ) -> anyhow::Result<Self> {
        global_init();
        let main_cfg = &mut cfg.main_cfg;

        if main_cfg.cmd_args.test_mode {
            logger_init(
                main_cfg.cmd_args.test_mode,
                Some(&main_cfg.debug),
                std::io::sink,
            );
        } else {
            logger_init(
                main_cfg.cmd_args.test_mode,
                Some(&main_cfg.debug),
                std::io::stdout,
            );
        }
        // start maintain mode
        shared_state::set_maintaining(main_cfg.cmd_args.maintaining);
        // Set up shared state
        shared_state::set_auto_clean_duration(main_cfg.auto_clean_duration);
        shared_state::set_file_save_days(main_cfg.file_save_days);
        shared_state::set_friends_number_limit(main_cfg.friends_number_limit);

        if let Some(new_ip) = parser.ip {
            main_cfg.ip = new_ip;
        }

        // port
        let port = match parser.port {
            None => main_cfg.port,
            Some(port) => port,
        };
        let addr: SocketAddr = format!("{}:{}", &main_cfg.ip, port).parse()?;
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

    /// Start the server and run forever.
    ///
    /// This function will start the http server, rpc server, file system, shutdown signal listener,
    /// and cmd from stdin (if enabled). It will also register the service to the registry.
    ///
    /// The function will return an error if any of the above fails.
    ///
    /// You can use `get_abort_handle` to get the shutdown handle and use it to stop the server.
    ///
    /// The server will not exit until all the tasks are finished.
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
        tracing::info!("Start to register service to registry");
        register_service(&cfg.registry).await?;
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
