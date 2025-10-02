#![feature(decl_macro)]
#![feature(duration_constructors)]

pub mod config;
pub mod db;
pub mod helper;
pub mod httpserver;
pub mod matrix;
pub mod process;
pub mod rabbitmq;
mod server;
mod shared_state;
pub mod webrtc;

use crate::config::{Cfg, MainCfg};
use crate::httpserver::Launcher;
use base::consts::{self, LOG_OUTPUT_DIR, SERVER_INFO_PATH};
use base::database::DbPool;
use base::log;
use base::shutdown::ShutdownSdr;
use base::wrapper::JobSchedulerWrapper;
use clap::Parser;
use dashmap::DashMap;
use db::file_storage;
use parking_lot::{Mutex, Once};
use process::error_msg::MAINTAINING;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};
use tracing::info;

#[derive(Debug, Parser, Default, Clone)]
#[command(author = "SkyUOI", version = base::build::VERSION, about = "The Server of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "binding port")]
    pub port: Option<u16>,
    #[arg(long, help = "binding ip")]
    pub ip: Option<String>,
    #[command(flatten)]
    pub shared_cfg: ParserCfg,
    #[arg(long, help = "server info file path", default_value = SERVER_INFO_PATH)]
    pub server_info: PathBuf,
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

#[derive(Debug, Serialize, Deserialize)]
struct ServerInfo {
    unique_id: uuid::Uuid,
    machine_id: u64,
    secret: String,
    server_name: String,
    version: u64,
}

const SECRET_LEN: usize = 32;

pub static ARG_PARSER: LazyLock<ArgsParser> = LazyLock::new(ArgsParser::parse);

static SERVER_INFO: LazyLock<ServerInfo> = LazyLock::new(|| {
    info!("server info path: {}", ARG_PARSER.server_info.display());

    let state = ARG_PARSER.server_info.exists();
    let server_name = || -> String {
        #[cfg(feature = "meaningful_name")]
        {
            let faker = fake::faker::name::en::Name();
            use fake::Fake;
            faker.fake()
        }
        #[cfg(not(feature = "meaningful_name"))]
        {
            helper::generate_random_string(10)
        }
    };
    if state {
        let origin_info: serde_json::Value =
            serde_json::from_reader(&fs::File::open(&ARG_PARSER.server_info).unwrap())
                .expect("read server info error");
        if let serde_json::Value::Number(version) = &origin_info["version"] {
            let current_version = version.as_i64().unwrap() as u64;
            let info = match current_version.cmp(&consts::SERVER_INFO_JSON_VERSION) {
                Ordering::Less => {
                    tracing::info!("server info is updating now");
                    let unique_id: uuid::Uuid = origin_info
                        .get("unique_id")
                        .map_or_else(uuid::Uuid::new_v4, |id| {
                            serde_json::from_str(&id.to_string()).unwrap()
                        });
                    let machine_id: u64 = origin_info.get("machine_id").map_or_else(
                        || rand::rng().random_range(0..(1024 - 1)),
                        |machine_id| serde_json::from_str(&machine_id.to_string()).unwrap(),
                    );
                    let secret: String = origin_info.get("secret").map_or_else(
                        || helper::generate_random_string(SECRET_LEN),
                        |secret| serde_json::from_str(&secret.to_string()).unwrap(),
                    );
                    let server_name: String = origin_info
                        .get("server_name")
                        .map_or_else(server_name, |server_name| {
                            serde_json::from_str(&server_name.to_string()).unwrap()
                        });
                    let version = current_version;
                    let info = ServerInfo {
                        unique_id,
                        machine_id,
                        secret,
                        server_name,
                        version,
                    };
                    // first backup the old server info
                    let backup_path = ARG_PARSER.server_info.with_extension("old");
                    fs::rename(&ARG_PARSER.server_info, &backup_path).unwrap();
                    info!("backup server info to {}", backup_path.display());
                    let mut f = fs::File::create(&ARG_PARSER.server_info).unwrap();
                    serde_json::to_writer_pretty(&mut f, &info).unwrap();
                    info
                }
                Ordering::Equal => serde_json::from_value(origin_info).unwrap(),
                Ordering::Greater => {
                    panic!("server info version is too high");
                }
            };
            return info;
        } else {
            panic!(
                "Format Error: cannot find version in \"{}\"",
                ARG_PARSER.server_info.display()
            );
        }
    }
    tracing::info!("Create server info file");

    let mut f = fs::File::create(&ARG_PARSER.server_info).unwrap();
    let id: u64 = rand::rng().random_range(0..(1024 - 1));
    let server_name = server_name();
    let info = ServerInfo {
        unique_id: uuid::Uuid::new_v4(),
        machine_id: id,
        secret: helper::generate_random_string(SECRET_LEN),
        server_name,
        version: consts::SERVER_INFO_JSON_VERSION,
    };
    serde_json::to_writer_pretty(&mut f, &info).unwrap();
    info
});

fn clear() -> anyhow::Result<()> {
    let dir_path = Path::new(LOG_OUTPUT_DIR);
    if !dir_path.exists() {
        tracing::warn!("try clear log but not found");
        return Ok(());
    }
    fs::remove_dir_all(dir_path)?;
    fs::create_dir(dir_path)?;
    Ok(())
}

/// global init can be called many times,but only the first time will be effective
fn global_init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        color_eyre::install().ok();
    })
}

pub struct Application {
    pub shared: Arc<SharedData>,
    pub http_launcher: Option<Launcher>,
    pub pool: DbPool,
    pub rabbitmq: deadpool_lapin::Pool,
    /// for shutting down server fully,you shouldn't use handle.abort() to do this
    abort_sender: ShutdownSdr,
    pub started_notify: Arc<tokio::sync::Notify>,
}

/// shared data along the whole application
#[derive(Debug)]
pub struct SharedData {
    pub cfg: Cfg,
    pub verify_record: DashMap<String, Arc<tokio::sync::Notify>>,
    maintaining: Mutex<bool>,
    sched: tokio::sync::Mutex<JobSchedulerWrapper>,
}

impl SharedData {
    pub fn set_maintaining(&self, maintaining: bool) {
        *self.maintaining.lock() = maintaining;
        info!("set maintaining:{}", maintaining);
    }

    pub fn get_maintaining(&self) -> bool {
        *self.maintaining.lock()
    }

    #[allow(clippy::result_large_err)]
    pub fn convert_maintaining_into_grpc_status(&self) -> Result<(), tonic::Status> {
        if self.get_maintaining() {
            Err(tonic::Status::unavailable(MAINTAINING))
        } else {
            Ok(())
        }
    }
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

impl Application {
    /// Builds a new `Application` instance.
    ///
    /// This function will set up the log system, shared state, connect to the database,
    /// and connect to Redis.
    /// The `parser` argument is used to override some
    /// configuration if specified.
    /// The `cfg` argument is the configuration to be used.
    ///
    /// # Arguments
    ///
    /// * `parser` - The parsed command line arguments.
    /// * `cfg` - The configuration to be used.
    /// * `email_client` - The email client to be used.
    ///   If `None`, the email client
    ///   will be ignored.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Application` instance if successful, or an error
    /// if it fails to set up the log system, shared state, connect to the database, or
    /// connect to Redis.
    pub async fn build(parser: ArgsParser, mut cfg: Cfg) -> anyhow::Result<Self> {
        global_init();
        let main_cfg = &mut cfg.main_cfg;

        if main_cfg.cmd_args.test_mode {
            log::logger_init(
                main_cfg.cmd_args.test_mode,
                Some(&main_cfg.debug),
                std::io::sink,
                consts::OURCHAT_LOG_PREFIX,
            );
        } else {
            log::logger_init(
                main_cfg.cmd_args.test_mode,
                Some(&main_cfg.debug),
                std::io::stdout,
                consts::OURCHAT_LOG_PREFIX,
            );
        }
        info!("Machine ID: {}", SERVER_INFO.machine_id);
        let maintaining = main_cfg.cmd_args.maintaining;
        // Set up shared state
        shared_state::set_friends_number_limit(main_cfg.friends_number_limit);

        if let Some(new_ip) = parser.ip {
            cfg.http_cfg.ip = new_ip;
        }

        // port
        let port = match parser.port {
            None => cfg.http_cfg.port,
            Some(port) => port,
        };
        cfg.http_cfg.port = port;

        let abort_sender = ShutdownSdr::new(None);
        db::init_db_system();
        // connect to db
        let db_pool = DbPool::build(&cfg.db_cfg, &cfg.redis_cfg, true).await?;
        db_pool.init().await?;
        // connect to rabbitmq
        let rmq_pool = cfg.rabbitmq_cfg.build().await?;
        if cfg.main_cfg.unique_instance() {
            rabbitmq::init(&rmq_pool).await?;
        }

        let sched = tokio::sync::Mutex::new(JobSchedulerWrapper::new(
            tokio_cron_scheduler::JobScheduler::new().await?,
        ));
        log::add_clean_to_scheduler(
            consts::OURCHAT_LOG_PREFIX,
            cfg.main_cfg.lop_keep,
            cfg.main_cfg.log_clean_duration,
            sched.lock().await,
        )
        .await?;
        // init http server
        let http_launcher = Launcher::build_from_config(&mut cfg).await?;

        // init some regular tasks
        crate::process::webrtc::clean_rooms(
            cfg.main_cfg.voip.empty_room_keep_duration,
            db_pool.clone(),
            sched.lock().await,
        )
        .await?;

        Ok(Self {
            http_launcher: Some(http_launcher),
            shared: Arc::new(SharedData {
                cfg,
                verify_record: DashMap::new(),
                maintaining: Mutex::new(maintaining),
                sched,
            }),
            pool: db_pool,
            abort_sender,
            started_notify: Arc::new(tokio::sync::Notify::new()),
            rabbitmq: rmq_pool,
        })
    }

    pub fn get_port(&self) -> u16 {
        self.shared.cfg.http_cfg.port
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
        info!("Starting server");
        let cfg = &self.shared.cfg.main_cfg;

        if cfg.cmd_args.clear {
            clear()?;
        }

        let grpc_builder = server::RpcServer::new(
            self.pool.clone(),
            self.shared.clone(),
            self.rabbitmq.clone(),
        );
        let grpc_service = grpc_builder.construct_grpc().await?;
        let mut launcher = self.http_launcher.take().unwrap();
        let shared_clone = self.shared.clone();
        let rabbitmq_clone = self.rabbitmq.clone();
        let pool_clone = self.pool.clone();

        let wait_http_setup = launcher.started_notify.clone();

        let shutdown_sdr = self.abort_sender.clone();
        let handle = tokio::spawn(async move {
            launcher
                .run_forever(
                    shared_clone,
                    rabbitmq_clone,
                    pool_clone,
                    grpc_service,
                    shutdown_sdr,
                )
                .await
        });

        wait_http_setup.notified().await;
        // Start the database file system
        file_storage::FileSys::new(self.pool.db_pool.clone(), self.shared.clone())
            .start()
            .await?;
        self.shared.sched.lock().await.start().await?;
        info!("Start to register service to registry");
        info!("Server started");
        self.started_notify.notify_waiters();
        match handle.await {
            Ok(result) => match result {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("server error:{:?}", e);
                }
            },
            Err(e) => {
                tracing::error!("server error when joining:{:?}", e);
            }
        };
        self.pool.close().await?;
        self.rabbitmq.close();
        self.shared.sched.lock().await.shutdown().await?;
        info!("Server exited");
        Ok(())
    }
}

#[ctor::ctor]
fn init() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
