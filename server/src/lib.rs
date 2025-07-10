#![feature(decl_macro)]
#![feature(duration_constructors)]

mod cryption;
pub mod db;
pub mod helper;
pub mod process;
pub mod rabbitmq;
mod server;
mod shared_state;
mod webrtc;

use anyhow::bail;
use base::consts::{self, CONFIG_FILE_ENV_VAR, LOG_OUTPUT_DIR, SERVER_INFO_PATH};
use base::database::DbPool;
use base::database::postgres::PostgresDbCfg;
use base::database::redis::RedisCfg;
use base::rabbitmq::RabbitMQCfg;
use base::setting::debug::DebugCfg;
use base::setting::tls::TlsConfig;
use base::setting::{Setting, UserSetting};
use base::shutdown::{ShutdownRev, ShutdownSdr};
use base::wrapper::JobSchedulerWrapper;
use base::{log, setting};
use clap::Parser;
use dashmap::DashMap;
use db::file_storage;
use futures_util::future::join_all;
use parking_lot::{Mutex, Once};
use process::error_msg::MAINTAINING;
use rand::Rng;
use serde::{Deserialize, Serialize};
use size::Size;
use std::cmp::Ordering;
use std::time::Duration;
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};
use tokio::task::JoinHandle;
use tracing::info;
use utils::merge_json;

#[derive(Debug, Parser, Default)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainCfg {
    #[serde(default = "consts::default_ip")]
    pub ip: String,
    pub redis_cfg: PathBuf,
    pub db_cfg: PathBuf,
    pub rabbitmq_cfg: PathBuf,
    pub user_setting: PathBuf,
    #[serde(default = "consts::default_port")]
    pub port: u16,
    #[serde(default = "consts::default_http_port")]
    pub http_port: u16,
    #[serde(default = "consts::default_clear_interval")]
    pub auto_clean_duration: croner::Cron,
    #[serde(default = "consts::default_file_save_time", with = "humantime_serde")]
    pub file_save_time: Duration,
    #[serde(default = "consts::default_user_files_store_limit")]
    pub user_files_limit: Size,
    #[serde(default = "consts::default_friends_number_limit")]
    pub friends_number_limit: u32,
    #[serde(default = "consts::default_files_storage_path")]
    pub files_storage_path: PathBuf,
    #[serde(
        default = "consts::default_verification_expire_time",
        with = "humantime_serde"
    )]
    pub verification_expire_time: Duration,
    #[serde(
        default = "consts::default_user_defined_status_expire_time",
        with = "humantime_serde"
    )]
    pub user_defined_status_expire_time: Duration,
    #[serde(
        default = "consts::default_log_clean_duration",
        with = "humantime_serde"
    )]
    pub log_clean_duration: Duration,
    #[serde(default = "consts::default_log_keep", with = "humantime_serde")]
    pub lop_keep: Duration,
    #[serde(default = "consts::default_single_instance")]
    pub single_instance: bool,
    #[serde(default = "consts::default_leader_node")]
    pub leader_node: bool,
    #[serde(
        default = "consts::default_room_key_duration",
        with = "humantime_serde"
    )]
    pub room_key_duration: Duration,
    pub password_hash: PasswordHash,
    pub db: DbArgCfg,
    pub debug: DebugCfg,
    pub tls: TlsConfig,

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
pub struct DbArgCfg {
    #[serde(default = "consts::default_fetch_msg_page_size")]
    pub fetch_msg_page_size: u64,
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
        // read a config file
        let mut cfg: serde_json::Value = setting::read_a_config(&cfg_path)
            .expect("Failed to build config")
            .try_deserialize()
            .expect("Wrong config file structure");
        let mut configs_list = vec![cfg_path];
        for i in iter {
            let i = i.into();
            let merge_cfg: serde_json::Value = setting::read_a_config(&i)
                .expect("Failed to build config")
                .try_deserialize()
                .expect("Wrong config file structure");
            cfg = merge_json(cfg, merge_cfg);
            configs_list.push(i);
        }
        let mut cfg: MainCfg = serde_json::from_value(cfg).expect("Failed to deserialize config");
        cfg.cmd_args.config = configs_list;
        // convert the path relevant to the config file to a path relevant to the directory
        cfg.convert_to_abs_path()?;
        Ok(cfg)
    }

    /// Return the protocol used for HTTP requests, either "http" or "https", based on the ssl field.
    pub fn protocol_http(&self) -> String {
        if self.tls.enable {
            "https".to_string()
        } else {
            "http".to_string()
        }
    }

    pub fn unique_instance(&self) -> bool {
        self.leader_node || self.single_instance
    }
}

#[derive(Debug, Clone)]
pub struct Cfg {
    pub main_cfg: MainCfg,
    pub db_cfg: PostgresDbCfg,
    pub redis_cfg: RedisCfg,
    pub rabbitmq_cfg: RabbitMQCfg,
    pub user_setting: UserSetting,
}

impl Cfg {
    pub fn new(main_cfg: MainCfg) -> anyhow::Result<Self> {
        let db_cfg = PostgresDbCfg::build_from_path(&main_cfg.db_cfg)?;
        let redis_cfg = RedisCfg::build_from_path(&main_cfg.redis_cfg)?;
        let rabbitmq_cfg = RabbitMQCfg::build_from_path(&main_cfg.rabbitmq_cfg)?;
        let user_setting = UserSetting::build_from_path(&main_cfg.user_setting)?;
        Ok(Self {
            main_cfg,
            db_cfg,
            redis_cfg,
            rabbitmq_cfg,
            user_setting,
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
    /// convert config paths to absolute path
    ///
    /// the base path is the first config file's parent directory
    ///
    /// the paths to be converted:
    ///
    /// - `redis_cfg`
    /// - `db_cfg`
    /// - `rabbitmq_cfg`
    /// - `user_setting`
    fn convert_to_abs_path(&mut self) -> anyhow::Result<()> {
        let full_basepath = self
            .cmd_args
            .config
            .first()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()?;
        self.redis_cfg = utils::resolve_relative_path(&full_basepath, Path::new(&self.redis_cfg))?;
        self.db_cfg = utils::resolve_relative_path(&full_basepath, Path::new(&self.db_cfg))?;
        self.rabbitmq_cfg =
            utils::resolve_relative_path(&full_basepath, Path::new(&self.rabbitmq_cfg))?;
        self.user_setting =
            utils::resolve_relative_path(&full_basepath, Path::new(&self.user_setting))?;
        Ok(())
    }
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

static SERVER_INFO: LazyLock<ServerInfo> = LazyLock::new(|| {
    // Extract only server-info argument
    let path = clap::Command::new("ourchat")
        .arg(
            clap::Arg::new("server-info")
                .long("server-info")
                .help("server info file path")
                .default_value(SERVER_INFO_PATH),
        )
        .ignore_errors(true)
        .get_matches()
        .get_one::<String>("server-info")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(SERVER_INFO_PATH));

    info!("server info path: {}", path.display());

    let state = path.exists();
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
            serde_json::from_reader(&fs::File::open(&path).unwrap())
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
                        || rand::thread_rng().gen_range(0..(1024 - 1)),
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
                    let backup_path = path.with_extension("old");
                    fs::rename(&path, &backup_path).unwrap();
                    info!("backup server info to {}", backup_path.display());
                    let mut f = fs::File::create(&path).unwrap();
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
                path.display()
            );
        }
    }
    tracing::info!("Create server info file");

    let mut f = fs::File::create(&path).unwrap();
    let id: u64 = rand::thread_rng().gen_range(0..(1024 - 1));
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

/// This function listen to sigterm and ctrl-c signal. When the signal is received, it will call
/// `shutdown_all_tasks` to shut down all tasks and exit the process.
fn exit_signal(#[allow(unused_mut)] mut shutdown_sender: ShutdownSdr) -> anyhow::Result<()> {
    let mut shutdown_sender_clone = shutdown_sender.clone();
    #[cfg(not(windows))]
    tokio::spawn(async move {
        if let Some(()) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?
            .recv()
            .await
        {
            info!("Exit because of sigterm signal");
            shutdown_sender.shutdown_all_tasks().await?;
        }
        anyhow::Ok(())
    });
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Exit because of ctrl-c signal");
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

/// build websocket server
async fn start_server(
    addr: impl Into<SocketAddr>,
    db: DbPool,
    shared_data: Arc<SharedData>,
    rabbitmq: deadpool_lapin::Pool,
    shutdown_receiver: ShutdownRev,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let server = server::RpcServer::new(addr, db, shared_data, rabbitmq);
    let handle = tokio::spawn(async move { server.run(shutdown_receiver).await });
    Ok(handle)
}

/// global init can be called many times,but only the first time will be effective
fn global_init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        println!("Machine ID: {}", SERVER_INFO.machine_id);
        color_eyre::install().ok();
    })
}

pub struct Application {
    pub shared: Arc<SharedData>,
    pub pool: DbPool,
    pub rabbitmq: deadpool_lapin::Pool,
    server_addr: SocketAddr,
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
        let maintaining = main_cfg.cmd_args.maintaining;
        // Set up shared state
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
        base::log::add_clean_to_scheduler(
            consts::OURCHAT_LOG_PREFIX,
            cfg.main_cfg.lop_keep,
            cfg.main_cfg.log_clean_duration,
            sched.lock().await,
        )
        .await?;

        Ok(Self {
            shared: Arc::new(SharedData {
                cfg,
                verify_record: DashMap::new(),
                maintaining: Mutex::new(maintaining),
                sched,
            }),
            pool: db_pool,
            server_addr: addr,
            abort_sender,
            started_notify: Arc::new(tokio::sync::Notify::new()),
            rabbitmq: rmq_pool,
        })
    }

    pub fn get_port(&self) -> u16 {
        self.shared.cfg.main_cfg.port
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

        let mut handles = Vec::new();

        let handle = start_server(
            self.server_addr,
            self.pool.clone(),
            self.shared.clone(),
            self.rabbitmq.clone(),
            self.abort_sender.new_receiver("rpc server", "rpc server"),
        )
        .await?;
        handles.push(handle);

        // Start the database file system
        file_storage::FileSys::new(self.pool.db_pool.clone(), self.shared.clone())
            .start()
            .await?;
        // Start the shutdown signal listener
        exit_signal(self.abort_sender.clone())?;
        self.shared.sched.lock().await.start().await?;
        info!("Start to register service to registry");
        info!("Server started");
        self.started_notify.notify_waiters();
        join_all(handles).await.iter().for_each(|x| match x {
            Ok(result) => match result {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("server error:{:?}", e);
                }
            },
            Err(e) => {
                tracing::error!("server error when joining:{:?}", e);
            }
        });
        self.pool.close().await?;
        self.rabbitmq.close();
        self.shared.sched.lock().await.shutdown().await?;
        info!("Server exited");
        Ok(())
    }
}
