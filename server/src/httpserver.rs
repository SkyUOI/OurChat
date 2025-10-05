mod avatar;
mod status;
pub mod verify;

use crate::{Cfg, SharedData};
use anyhow::anyhow;
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use base::{
    database::DbPool,
    email_client::{EmailCfg, EmailSender},
    setting::tls::TlsConfig,
    shutdown::{ShutdownRev, ShutdownSdr},
};
use deadpool_lapin::lapin::options::{BasicAckOptions, BasicRejectOptions};
use http::{Method, Uri};
use rustls::{
    RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tokio::{select, signal};
use tokio_stream::StreamExt;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tracing::{debug, info, warn};

pub struct HttpServer {}

pub struct ServerRunningData {
    shared_data: Arc<SharedData>,
    rabbitmq: deadpool_lapin::Pool,
    db_pool: DbPool,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_forever(
        &mut self,
        listener: tokio::net::TcpListener,
        email_client: Option<EmailClientType>,
        running_data: ServerRunningData,
        grpc_service: tonic::service::Routes,
        shutdown_sdr: ShutdownSdr,
    ) -> anyhow::Result<()> {
        info!("Start building Server");
        let shared_data = running_data.shared_data;
        let db_pool = running_data.db_pool;
        let rabbitmq = running_data.rabbitmq;

        let enable_matrix = shared_data.cfg.http_cfg.enable_matrix;
        let cors = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::HeaderName::from_static("x-requested-with"),
            ]);
        let rate_governor_config = GovernorConfigBuilder::default()
            .burst_size(shared_data.cfg.http_cfg.rate_limit.num_of_burst_requests)
            .per_millisecond(
                shared_data
                    .cfg
                    .http_cfg
                    .rate_limit
                    .replenish_duration
                    .as_millis() as u64,
            )
            .finish()
            .unwrap();
        let rate_governor_limiter = rate_governor_config.limiter().clone();
        // background task to clean up
        // copy the example of tower_governor
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_mins(1)).await;
                info!(
                    "rate limiting storage size: {}",
                    rate_governor_limiter.len()
                );
                rate_governor_limiter.retain_recent();
            }
        });
        let v1 = axum::Router::new()
            .route("/status", get(status::status))
            .route_service(
                "/logo",
                tower_http::services::ServeFile::new(shared_data.cfg.http_cfg.logo_path.clone()),
            )
            .route("/avatar", get(avatar::avatar))
            .merge(verify::config().with_state(db_pool.clone()));
        let mut router: axum::Router = axum::Router::new()
            .nest("/v1", v1.with_state((db_pool.clone(), shared_data.clone())))
            .merge(grpc_service.into_axum_router())
            .layer(
                ServiceBuilder::new()
                    .layer(tower_http::trace::TraceLayer::new_for_http())
                    .layer(tower_http::trace::TraceLayer::new_for_grpc())
                    .layer(cors)
                    .layer(tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash()),
            );
        if shared_data.cfg.http_cfg.rate_limit.enable {
            info!("Http rate limit enabled");
            router = router.layer(GovernorLayer::new(rate_governor_config));
        } else {
            warn!("Http rate limit disabled");
        }
        if enable_matrix {
            info!("matrix api enabled");
            router = router
                .nest("/_matrix", crate::matrix::configure_matrix())
                .nest(
                    "/.well-known",
                    crate::matrix::route::wellknown::configure_route()
                        .with_state(shared_data.clone()),
                );
        }

        info!("Start creating rabbitmq consumer");
        let connection = rabbitmq.get().await?;
        debug!("Get connection to rabbitmq");
        let channel = connection.create_channel().await?;
        debug!("Get channel to rabbitmq");
        let rabbit_listen_rev =
            shutdown_sdr.new_receiver("rabbitmq verify", "listen to rabbitmq to get verify record");
        let shared_data_clone = shared_data.clone();
        tokio::spawn(async move {
            match Self::listen_rabbitmq(
                channel,
                db_pool,
                shared_data_clone,
                email_client,
                rabbit_listen_rev,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        });
        info!("Http server setup done");
        let mut rev = shutdown_sdr.new_receiver("http server", "http server");

        let running_server = async move {
            info!("Listening on {}", listener.local_addr().unwrap());
            if shared_data.cfg.http_cfg.tls.is_tls_on()? {
                let handle = axum_server::Handle::new();
                let handle_clone = handle.clone();
                tokio::spawn(async move { exit_signal_handle_wrapper(handle_clone).await });
                let mut config = self.load_rustls_config(shared_data)?;
                config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
                axum_server::from_tcp_rustls(
                    listener.into_std()?,
                    RustlsConfig::from_config(Arc::new(config)),
                )
                .handle(handle)
                .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
            } else {
                axum::serve(
                    listener,
                    router.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .with_graceful_shutdown(exit_signal())
                .await?;
            }
            anyhow::Ok(())
        };
        select! {
            _ = rev.wait_shutting_down() => {
                Ok(())
            }
            e = running_server => {
                e?;
                Ok(())
            }
        }
    }

    fn load_rustls_config(&self, cfg: Arc<SharedData>) -> anyhow::Result<ServerConfig> {
        let mut cert_store = RootCertStore::empty();
        CertificateDer::pem_file_iter(cfg.cfg.http_cfg.tls.ca_tls_cert_path.as_ref().unwrap())?
            .flatten()
            .for_each(|der| cert_store.add(der).unwrap());

        // let client_auth = WebPkiClientVerifier::builder(Arc::new(cert_store)).build()?;

        let key_der = PrivateKeyDer::from_pem_file(
            cfg.cfg.http_cfg.tls.server_key_cert_path.as_ref().unwrap(),
        )?;
        let cert_chain = CertificateDer::pem_file_iter(
            cfg.cfg.http_cfg.tls.server_tls_cert_path.as_ref().unwrap(),
        )?
        .flatten()
        .collect();
        Ok(ServerConfig::builder()
            // .with_client_cert_verifier(client_auth)
            .with_no_client_auth()
            .with_single_cert(cert_chain, key_der)?)
    }

    async fn listen_rabbitmq(
        mq_channel: deadpool_lapin::lapin::Channel,
        db_pool: DbPool,
        shared_data: Arc<SharedData>,
        email_client: Option<EmailClientType>,
        mut shutdown_rev: ShutdownRev,
    ) -> anyhow::Result<()> {
        let logic = async {
            debug!("Starting set channel");
            // TODO:add this to config file
            mq_channel
                .basic_qos(
                    70,
                    deadpool_lapin::lapin::options::BasicQosOptions::default(),
                )
                .await?;
            // Wait for the channel to be set
            let mut try_cnt = 0;
            let mut consumer = loop {
                match mq_channel
                    .basic_consume(
                        base::rabbitmq::http_server::VERIFY_QUEUE,
                        "http_server",
                        deadpool_lapin::lapin::options::BasicConsumeOptions::default(),
                        deadpool_lapin::lapin::types::FieldTable::default(),
                    )
                    .await
                {
                    Ok(c) => {
                        break c;
                    }
                    Err(e) => {
                        tracing::error!("try {} to get consumer failed:{}", try_cnt, e);
                        if try_cnt == 9 {
                            return Err(anyhow!(e));
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(3)).await;
                try_cnt += 1;
            };
            debug!("Starting to consume verification");
            while let Some(data) = consumer.next().await {
                let delivery = match data {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!("{}", e);
                        continue;
                    }
                };
                let verify_record = serde_json::from_slice::<
                    base::rabbitmq::http_server::VerifyRecord,
                >(&delivery.data[..])?;
                match verify::verify_client(
                    &db_pool,
                    &email_client,
                    verify_record.clone(),
                    &shared_data,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{}", e);
                    }
                }
                let redis_conn = db_pool.redis_pool.clone();
                let verify_email_expiry = shared_data.cfg.user_setting.verify_email_expiry;
                tokio::spawn(async move {
                    tokio::time::sleep(verify_email_expiry).await;
                    let reject = async {
                        match delivery.reject(BasicRejectOptions { requeue: false }).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("{}", e);
                            }
                        }
                    };
                    let status = match verify::check_token_exist_and_del_token(
                        &verify_record.token,
                        &redis_conn,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(e) => {
                            reject.await;
                            tracing::error!("check token error:{e}");
                            return;
                        }
                    };
                    if status {
                        reject.await;
                    } else {
                        match delivery.ack(BasicAckOptions::default()).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("ack verify failed:{}", e);
                            }
                        }
                    }
                });
            }
            anyhow::Ok(())
        };
        select! {
            ret = logic => {
                ret
            }
            _ = shutdown_rev.wait_shutting_down() => {
                Ok(())
            }
        }
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HttpCfg {
    #[serde(default = "base::consts::default_ip")]
    pub ip: String,
    #[serde(default = "base::consts::default_port")]
    pub port: u16,
    pub logo_path: PathBuf,
    pub default_avatar_path: PathBuf,
    #[serde(default = "base::consts::default_http_run_migration")]
    pub run_migration: bool,
    #[serde(default = "base::consts::default_enable_matrix")]
    pub enable_matrix: bool,
    #[serde(
        default = "base::consts::default_log_clean_duration",
        with = "humantime_serde"
    )]
    pub log_clean_duration: Duration,
    #[serde(default = "base::consts::default_log_keep", with = "humantime_serde")]
    pub lop_keep: Duration,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub rate_limit: RateLimitCfg,
    pub email_cfg: Option<PathBuf>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RateLimitCfg {
    #[serde(default = "base::consts::default_rate_limit_enable")]
    pub enable: bool,
    #[serde(default = "base::consts::default_rate_limit_burst")]
    pub num_of_burst_requests: u32,
    #[serde(
        default = "base::consts::default_rate_limit_replenish_duration",
        with = "humantime_serde"
    )]
    pub replenish_duration: Duration,
}

impl Default for RateLimitCfg {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}

impl base::setting::Setting for HttpCfg {}
impl base::setting::PathConvert for HttpCfg {
    fn convert_to_abs_path(&mut self, full_basepath: &Path) -> anyhow::Result<()> {
        self.logo_path = utils::resolve_relative_path(full_basepath, &self.logo_path)?;
        self.email_cfg = match &self.email_cfg {
            Some(email_cfg) => Some(utils::resolve_relative_path(full_basepath, email_cfg)?),
            None => None,
        };
        Ok(())
    }
}

impl HttpCfg {
    pub fn protocol_http(&self) -> &'static str {
        if self.tls.enable { "https" } else { "http" }
    }

    pub fn base_url(&self) -> Uri {
        format!("{}://{}:{}", self.protocol_http(), self.ip, self.port)
            .parse()
            .unwrap()
    }

    pub fn domain(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug)]
pub struct Launcher {
    pub email_client: Option<EmailClientType>,
    pub tcplistener: Option<tokio::net::TcpListener>,
    pub started_notify: Arc<tokio::sync::Notify>,
}

pub type EmailClientType = Box<dyn EmailSender>;

impl Launcher {
    pub async fn build_from_config(cfg: &mut Cfg) -> anyhow::Result<Self> {
        let email_client: Option<Box<dyn EmailSender>> = match &cfg.http_cfg.email_cfg {
            Some(email_cfg) => {
                let email_cfg = EmailCfg::build_from_path(email_cfg)?;
                let email_client = email_cfg.build_email_client()?;
                Some(Box::new(email_client))
            }
            None => None,
        };
        let http_listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &cfg.http_cfg.ip, cfg.http_cfg.port))
                .await?;
        // deal with port 0
        cfg.http_cfg.port = http_listener.local_addr()?.port();
        let started_notify = Arc::new(tokio::sync::Notify::new());
        Ok(Self {
            email_client,
            tcplistener: Some(http_listener),
            started_notify,
        })
    }

    pub async fn run_forever(
        &mut self,
        shared_data: Arc<SharedData>,
        rabbitmq_pool: deadpool_lapin::Pool,
        db_pool: DbPool,
        grpc_service: tonic::service::Routes,
        abort_sender: ShutdownSdr,
    ) -> anyhow::Result<()> {
        let mut server = HttpServer::new();

        info!("Starting http server");
        let tcplistener = self.tcplistener.take().unwrap();
        let email_client: Option<Box<dyn EmailSender>> = self.email_client.take();
        let http_server = tokio::spawn(async move {
            server
                .run_forever(
                    tcplistener,
                    email_client,
                    ServerRunningData {
                        shared_data,
                        rabbitmq: rabbitmq_pool,
                        db_pool,
                    },
                    grpc_service,
                    abort_sender,
                )
                .await
        });
        info!("Started http server");
        info!("Sending started notification");
        self.started_notify.notify_waiters();
        info!("Http Server started");
        http_server.await??;
        Ok(())
    }
}

async fn exit_signal() {
    let ctrl_c = async {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Exit because of ctrl-c signal");
            }
            Err(err) => {
                tracing::error!("Unable to listen to ctrl-c signal:{}", err);
            }
        }
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        info!("Exit because of sigterm signal");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn exit_signal_handle_wrapper(handle: axum_server::Handle) {
    exit_signal().await;
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}
