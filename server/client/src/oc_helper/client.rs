use crate::helper::rabbitmq::{create_random_vhost, delete_vhost};
use crate::oc_helper::Clients;
use crate::oc_helper::TestSession;
use crate::oc_helper::user::{TestUser, TestUserShared};
use base::consts::{CONFIG_FILE_ENV_VAR, ID, OCID, SessionID};
use base::database::DbPool;
use base::shutdown::ShutdownSdr;
use migration::predefined::PredefinedRoles;
use pb::service::auth::v1::auth_service_client::AuthServiceClient;
use pb::service::basic::v1::basic_service_client::BasicServiceClient;
use pb::service::basic::v1::{GetIdRequest, TimestampRequest};
use pb::service::ourchat::v1::our_chat_service_client::OurChatServiceClient;
use pb::time::TimeStampUtc;
use sea_orm::TransactionTrait;
use server::config::Cfg;
use server::db::session::{BanStatus, MuteStatus, user_banned_status, user_muted_status};
use server::helper::get_available_port;
use server::{Application, ArgsParser, ParserCfg, SharedData, helper, process};
use sqlx::migrate::MigrateDatabase;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tonic::codegen::InterceptedService;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

pub type OCClient = OurChatServiceClient<
    InterceptedService<
        Channel,
        Box<
            dyn FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>
                + Send
                + Sync,
        >,
    >,
>;

/// A test client
///
/// # Details
/// Some members are Option because you can construct a TestApp by connecting to an existing server or not
#[derive(Clone)]
pub struct TestApp {
    pub db_url: String,
    pub app_shared: Arc<SharedData>,
    pub db_pool: DbPool,
    pub rabbitmq_pool: deadpool_lapin::Pool,
    pub owned_users: Vec<Arc<tokio::sync::Mutex<TestUser>>>,
    pub app_config: Cfg,
    pub rmq_vhost: String,
    pub core: ClientCore,
    pub http_client: reqwest::Client,

    has_dropped: bool,
    server_drop_handle: ShutdownSdr,
    pub should_drop_db: bool,
    pub should_drop_vhost: bool,
}

impl TestApp {
    pub fn basic_service(&mut self) -> &mut BasicServiceClient<Channel> {
        &mut self.core.clients.basic
    }
}

trait TestAppTrait {
    fn test() -> Self;
}

impl TestAppTrait for ArgsParser {
    fn test() -> Self {
        Self {
            port: Some(get_available_port()),
            shared_cfg: ParserCfg {
                test_mode: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientCore {
    pub port: u16,
    pub clients: Clients,
    pub rpc_url: String,
    pub enable_ssl: bool,
}

impl ClientCore {
    pub async fn new(cfg: ClientCoreConfig) -> anyhow::Result<Self> {
        let url_without_scheme = format!("{}:{}", cfg.ip, cfg.port);
        let enabled_tls = match cfg.enable_ssl {
            Some(data) => data,
            None => utils::http::test_and_get_http_status(&url_without_scheme).await?,
        };

        let remote_url = format!(
            "{}://{}",
            if enabled_tls { "https" } else { "http" },
            url_without_scheme
        );
        Ok(Self {
            port: cfg.port,
            rpc_url: remote_url.clone(),
            clients: Clients {
                auth: AuthServiceClient::connect(remote_url.clone()).await?,
                basic: BasicServiceClient::connect(remote_url.clone()).await?,
            },
            enable_ssl: enabled_tls,
        })
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ClientCoreConfig {
    ip: String,
    port: u16,
    enable_ssl: Option<bool>,
}

impl TestApp {
    pub fn get_test_config() -> anyhow::Result<ConfigWithArgs> {
        let mut args = ArgsParser::test();
        // Set the configuration file path explicitly for tests
        if let Ok(config_path) = std::env::var(CONFIG_FILE_ENV_VAR) {
            args.shared_cfg.config = vec![std::path::PathBuf::from(config_path)];
        }
        let mut config = server::get_configuration(args.shared_cfg.config.clone())?;
        config.http_cfg.rate_limit.enable = false;
        Ok((config, args))
    }

    pub async fn new_with_launching_instance() -> anyhow::Result<Self> {
        Self::new_with_launching_instance_custom_cfg(Self::get_test_config()?, |_| {}).await
    }

    /// # Example
    ///
    /// ```ignore
    /// let (mut config, args) = TestApp::get_test_config().unwrap();
    /// let user_files_limit = Size::from_mebibytes(10);
    /// config.main_cfg.user_files_limit = user_files_limit;
    /// let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args))
    ///     .await
    ///     .unwrap();
    /// ```
    pub async fn new_with_launching_instance_custom_cfg(
        (mut server_config, args): ConfigWithArgs,
        process_config: impl FnOnce(&mut Application),
    ) -> anyhow::Result<Self> {
        // should create a different database for each test
        let db = uuid::Uuid::new_v4().to_string();
        server_config.db_cfg.db = db;
        // Create a new virtual host for each test
        let vhost = create_random_vhost(
            &reqwest::Client::new(),
            &server_config.rabbitmq_cfg.manage_url().unwrap(),
        )
        .await?;
        server_config.rabbitmq_cfg.vhost = vhost.clone();
        let db_url = server_config.db_cfg.url();
        let mut application = Application::build(args, server_config.clone()).await?;
        process_config(&mut application);
        let port = application.get_port();
        server_config.http_cfg.port = port;
        let abort_handle = application.get_abort_handle();
        let shared = application.shared.clone();
        let db_pool = application.pool.clone();
        let rabbitmq_pool = application.rabbitmq.clone();

        let notifier = application.started_notify.clone();
        tokio::spawn(async move {
            application.run_forever().await.unwrap();
        });

        notifier.notified().await;
        let rpc_url = format!(
            "{}://localhost:{}",
            server_config.http_cfg.protocol_http(),
            port
        );
        let mut connected_channel = Endpoint::from_shared(rpc_url.clone())?;
        let enabled_tls = server_config.http_cfg.tls.is_tls_on()?;
        if enabled_tls {
            let client_cert = std::fs::read_to_string(
                server_config
                    .http_cfg
                    .tls
                    .client_tls_cert_path
                    .clone()
                    .unwrap(),
            )?;
            let client_key = std::fs::read_to_string(
                server_config
                    .http_cfg
                    .tls
                    .client_key_cert_path
                    .clone()
                    .unwrap(),
            )?;
            let client_identity = Identity::from_pem(client_cert.clone(), client_key);
            let server_ca_cert = std::fs::read_to_string(
                server_config.http_cfg.tls.ca_tls_cert_path.clone().unwrap(),
            )?;
            let server_root_ca = Certificate::from_pem(server_ca_cert);
            let tls = ClientTlsConfig::new()
                .ca_certificate(server_root_ca)
                .identity(client_identity);
            connected_channel = Endpoint::from_shared(rpc_url.clone())?.tls_config(tls)?;
        }
        let connected_channel = connected_channel.connect().await?;

        // Construct http client
        let mut http_client = reqwest::Client::builder().timeout(Duration::from_secs(2));
        if shared.cfg.http_cfg.tls.is_tls_on()? {
            // TODO: remove the danger_accept_invalid_certs (I think it is a bug of reqwest)
            let pem =
                tokio::fs::read(shared.cfg.http_cfg.tls.ca_tls_cert_path.as_ref().unwrap()).await?;
            let cert = reqwest::Certificate::from_pem(&pem)?;
            http_client = http_client
                .add_root_certificate(cert)
                .danger_accept_invalid_certs(true)
        }
        let http_client = http_client.build()?;

        let obj = TestApp {
            http_client,
            db_url,
            server_drop_handle: abort_handle,
            has_dropped: false,
            should_drop_db: true,
            app_shared: shared,
            owned_users: vec![],
            db_pool,
            rabbitmq_pool,
            core: ClientCore {
                port,
                clients: Clients {
                    auth: AuthServiceClient::new(connected_channel.clone()),
                    basic: BasicServiceClient::new(connected_channel),
                },
                rpc_url: rpc_url.clone(),
                enable_ssl: enabled_tls,
            },
            app_config: server_config,
            rmq_vhost: vhost,
            should_drop_vhost: true,
        };

        Ok(obj)
    }

    pub async fn async_drop(&mut self) {
        tracing::info!("async_drop called");
        for i in &self.owned_users {
            i.lock().await.async_drop().await;
        }
        self.server_drop_handle.shutdown_all_tasks().await.unwrap();
        tracing::info!("shutdown message sent");

        tracing::info!("app shut down");
        tokio::time::sleep(Duration::from_secs(1)).await;
        if self.should_drop_db {
            match sqlx::Postgres::drop_database(&self.db_url).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("failed to drop the database: {}", e);
                }
            }
            tracing::info!("db deleted");
        }

        if self.should_drop_vhost {
            delete_vhost(
                &reqwest::Client::new(),
                &self.app_config.rabbitmq_cfg.manage_url().unwrap(),
                &self.rmq_vhost,
            )
            .await
            .unwrap();
        }
        self.has_dropped = true;
    }

    pub async fn new_user(&mut self) -> anyhow::Result<TestUserShared> {
        let user = Arc::new(tokio::sync::Mutex::new(TestUser::random(&self.core).await));
        if self.app_config.http_cfg.tls.is_tls_on()? {
            user.lock().await.tls = self.app_config.http_cfg.tls.clone();
        }
        user.lock().await.register().await?;
        self.owned_users.push(user.clone());
        Ok(user)
    }

    /// Creates a new session at the database level with the specified number of users and session name.
    ///
    /// This function generates a session ID and creates a new session in the database with the given
    /// parameters. It also registers new users and adds them to the session as members.
    /// The first user in the vector of users will be the owner of the session.
    /// The rest of the users will be added as members.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of users to be created and added to the session.
    /// * `name` - The name of the session to be created.
    ///
    /// # Returns
    ///
    /// * `anyhow::Result<(Vec<TestUserShared>, TestSession)>` - A tuple containing a vector of
    ///   newly created and registered users and the created TestSession object.
    ///
    /// # Errors
    ///
    /// This function will return an error if user registration or session creation fails.
    pub async fn new_session_db_level(
        &mut self,
        n: usize,
        name: impl Into<String>,
        e2ee_on: bool,
    ) -> anyhow::Result<(Vec<TestUserShared>, TestSession)> {
        let mut users = Vec::with_capacity(n);
        for _ in 0..n {
            users.push(self.new_user().await?);
        }
        // create a group in database level
        let session_id = helper::generate_session_id()?;
        // then will join to session and add size column
        process::db::create_session_db(session_id, 0, name.into(), &self.db_pool.db_pool, e2ee_on)
            .await?;
        tracing::info!("create session:{}", session_id);
        let mut id_vec = vec![];
        for i in &users {
            let id = i.lock().await.id;
            id_vec.push(id);
        }
        tracing::debug!("id:{:?}", id_vec);
        let transaction = self.get_db_connection().begin().await?;
        process::db::join_in_session(
            session_id,
            id_vec[0],
            Some(PredefinedRoles::Owner.into()),
            &transaction,
        )
        .await?;
        process::db::batch_join_in_session(
            session_id,
            &id_vec[1..],
            Some(PredefinedRoles::Member.into()),
            &transaction,
        )
        .await?;
        transaction.commit().await?;
        Ok((users, TestSession::new(session_id)))
    }

    /// Helper function to create a friendship between two users
    pub async fn create_friendship(
        &mut self,
        user1_id: ID,
        user2_id: ID,
    ) -> anyhow::Result<SessionID> {
        // Simplified friendship creation - in real implementation this would
        // involve the full add_friend/accept_friend_invitation flow
        let transaction = self.get_db_connection().begin().await?;
        let session_id =
            server::db::friend::add_friend(user1_id, user2_id, None, None, &transaction).await?;
        transaction.commit().await?;
        Ok(session_id)
    }

    pub async fn change_role_db_level(
        _user_id: ID,
        _session_id: SessionID,
        _role_id: u64,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// # Warning
    /// Must request the server, shouldn't build a time from local by chrono, because some tests
    /// rely on this behavior
    pub async fn get_timestamp(&mut self) -> TimeStampUtc {
        let ret = self
            .basic_service()
            .timestamp(TimestampRequest {})
            .await
            .unwrap()
            .into_inner()
            .timestamp
            .unwrap();
        ret.try_into().unwrap()
    }

    pub async fn get_id(&mut self, ocid: OCID) -> Result<ID, tonic::Status> {
        let id: ID = self
            .basic_service()
            .get_id(GetIdRequest { ocid: ocid.0 })
            .await?
            .into_inner()
            .id
            .into();
        Ok(id)
    }

    /// Get the database connection.
    pub fn get_db_connection(&self) -> &sea_orm::DatabaseConnection {
        &self.db_pool.db_pool
    }

    pub fn get_redis_connection(&self) -> &deadpool_redis::Pool {
        &self.db_pool.redis_pool
    }

    pub async fn check_ban_status(
        &self,
        user: ID,
        session_id: SessionID,
    ) -> anyhow::Result<Option<BanStatus>> {
        let mut redis_connection = self.db_pool.redis_pool.get().await?;
        Ok(user_banned_status(user, session_id, &mut redis_connection).await?)
    }

    pub async fn check_mute_status(
        &self,
        user: ID,
        session_id: SessionID,
    ) -> anyhow::Result<Option<MuteStatus>> {
        let mut redis_connection = self.db_pool.redis_pool.get().await?;
        Ok(user_muted_status(user, session_id, &mut redis_connection).await?)
    }
}

impl TestApp {
    pub async fn ourchat_api_get(
        &self,
        name: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_get(format!("v1/{}", name.as_ref())).await
    }

    pub async fn matrix_api_get(
        &self,
        name: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_get(format!("_matrix/{}", name.as_ref())).await
    }

    pub async fn http_get(
        &self,
        url: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut base_url = self.app_config.http_cfg.base_url();
        if let Some(host) = base_url.host()
            && (host == "0.0.0.0" || host == "127.0.0.1")
        {
            let port = base_url.port_u16().unwrap_or(80);
            let mut parts = base_url.into_parts();
            parts.authority = Some(format!("localhost:{port}").parse().unwrap());
            base_url = http::Uri::from_parts(parts).unwrap();
        }
        self.http_client
            .get(format!("{}{}", base_url, url.as_ref()))
            .send()
            .await
    }

    pub async fn verify(&mut self, token: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.ourchat_api_get(format!("verify/confirm?token={token}"))
            .await
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this app");
        }
    }
}

pub type ConfigWithArgs = (Cfg, ArgsParser);
