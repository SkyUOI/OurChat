#![feature(asm_goto)]
#![feature(decl_macro)]
#![feature(duration_constructors)]

mod cmd;
pub mod component;
pub mod connection;
pub mod consts;
pub mod db;
mod entities;
pub mod requests;
mod server;
mod shared_state;
pub mod utils;

use crate::component::EmailClient;
use anyhow::bail;
use clap::Parser;
use cmd::CommandTransmitData;
use component::EmailSender;
use config::File;
use consts::{FileSize, STDIN_AVAILABLE};
use dashmap::DashMap;
use db::{DbCfg, DbType, MysqlDbCfg, SqliteDbCfg, file_storage};
use lettre::{AsyncSmtpTransport, transport::smtp::authentication::Credentials};
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
    sync::{broadcast, mpsc},
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
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
fn logger_init<Sink>(test_mode: bool, source: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    static INIT: OnceLock<Option<WorkerGuard>> = OnceLock::new();
    INIT.get_or_init(|| {
        let env = if test_mode {
            || EnvFilter::try_from_default_env().unwrap_or("trace".into())
        } else {
            || EnvFilter::try_from_default_env().unwrap_or("info".into())
        };
        let formatting_layer = fmt::layer().pretty().with_writer(source);

        let file_appender = tracing_appender::rolling::daily("log/", "ourchat");
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
}

/// build websocket server
async fn start_server(
    listener: TcpListener,
    db: DbPool,
    http_sender: HttpSender,
    shared_data: Arc<SharedData<impl EmailSender>>,
    shutdown_sender: ShutdownSdr,
    shutdown_receiver: broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    let mut server = server::Server::new(listener, db, http_sender, shared_data).await?;
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

pub struct Application<T: EmailSender> {
    pub shared: Arc<SharedData<T>>,
    pub pool: DbPool,
    server_listener: Option<TcpListener>,
    http_listener: Option<std::net::TcpListener>,
    /// for shutdowning server fully,you shouldn't use handle.abort() to do this
    abort_sender: ShutdownSdr,
}

#[derive(Debug, Clone)]
pub struct DbPool {
    db_pool: DatabaseConnection,
    redis_pool: deadpool_redis::Pool,
}

impl DbPool {
    async fn close(&mut self) -> anyhow::Result<()> {
        self.db_pool.clone().close().await?;
        self.redis_pool.close();
        Ok(())
    }
}

pub struct SharedData<T: EmailSender> {
    pub email_client: Option<T>,
    pub cfg: Cfg,
    pub verify_record: DashMap<String, Arc<tokio::sync::Notify>>,
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
        let addr = format!("{}:{}", &main_cfg.ip, port);
        let server_listener = TcpListener::bind(&addr).await?;
        main_cfg.port = server_listener.local_addr()?.port();

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

        // database type
        let db_type = match parser.db_type {
            None => main_cfg.db_type,
            Some(db_type) => match DbType::from_str(&db_type) {
                Ok(db_type) => db_type,
                Err(_) => bail!("Unknown database type. Only support mysql and sqlite"),
            },
        };
        main_cfg.db_type = db_type;

        // enable cmd
        let enable_cmd = match parser.enable_cmd {
            None => main_cfg.enable_cmd,
            Some(enable_cmd) => enable_cmd,
        };
        main_cfg.enable_cmd = enable_cmd;
        let (abort_sender, _) = broadcast::channel(20);
        // connect to database
        db::init_db_system(main_cfg.db_type);
        let db = db::connect_to_db(&db::get_db_url(&cfg.db_cfg)?).await?;
        db::init_db(&db).await?;
        // connect to redis
        let redis = db::connect_to_redis(&db::get_redis_url(&main_cfg.rediscfg)?).await?;

        Ok(Self {
            shared: Arc::new(SharedData {
                email_client,
                cfg,
                verify_record: DashMap::new(),
            }),
            pool: DbPool {
                redis_pool: redis,
                db_pool: db,
            },
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

        if cfg.cmd_args.clear {
            clear()?;
        }

        // Build http server
        let (handle, record_sender) = httpserver::HttpServer::new()
            .start(
                self.http_listener.take().unwrap(),
                self.pool.clone(),
                self.shared.clone(),
                self.abort_sender.subscribe(),
            )
            .await?;

        start_server(
            self.server_listener.take().unwrap(),
            self.pool.clone(),
            record_sender,
            self.shared.clone(),
            self.abort_sender.clone(),
            self.abort_sender.subscribe(),
        )
        .await?;
        // 启动数据库文件系统
        file_storage::FileSys::new(self.pool.db_pool.clone()).start(self.abort_sender.subscribe());
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
                self.pool.db_pool.clone(),
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
        self.pool.close().await?;
        tracing::info!("Server exited");
        Ok(())
    }
}
