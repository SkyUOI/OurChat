use crate::helper::rabbitmq::{create_random_vhost, delete_vhost};
use crate::oc_helper::Clients;
use crate::oc_helper::TestSession;
use crate::oc_helper::user::{TestUser, TestUserShared};
use base::consts::{ID, OCID, SessionID};
use base::database::DbPool;
use base::shutdown::ShutdownSdr;
use migration::m20241229_022701_add_role_for_session::PredefinedRoles;
use pb::service::auth::v1::auth_service_client::AuthServiceClient;
use pb::service::basic::v1::basic_service_client::BasicServiceClient;
use pb::service::basic::v1::{GetIdRequest, TimestampRequest};
use pb::service::ourchat::v1::our_chat_service_client::OurChatServiceClient;
use pb::time::{TimeStampUtc, from_google_timestamp};
use sea_orm::TransactionTrait;
use server::db::session::{BanStatus, MuteStatus, user_banned_status, user_muted_status};
use server::helper::get_available_port;
use server::{Application, ArgsParser, Cfg, ParserCfg, SharedData, helper, process};
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
    pub port: u16,
    pub db_url: String,
    pub app_shared: Option<Arc<SharedData>>,
    pub db_pool: Option<DbPool>,
    pub owned_users: Vec<Arc<tokio::sync::Mutex<TestUser>>>,
    pub clients: Clients,
    pub rpc_url: String,
    pub app_config: Cfg,
    pub rmq_vhost: String,

    has_dropped: bool,
    server_drop_handle: Option<ShutdownSdr>,
    pub should_drop_db: bool,
    pub should_drop_vhost: bool,
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

impl TestApp {
    pub fn get_test_config() -> anyhow::Result<ConfigWithArgs> {
        let args = ArgsParser::test();
        let config = server::get_configuration(args.shared_cfg.config.clone())?;
        Ok((config, args))
    }

    pub async fn new_with_launching_instance() -> anyhow::Result<Self> {
        Self::new_with_launching_instance_custom_cfg(Self::get_test_config()?).await
    }

    pub async fn new_with_launching_instance_custom_cfg(
        (mut server_config, args): ConfigWithArgs,
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
        let port = application.get_port();
        let abort_handle = application.get_abort_handle();
        let shared = application.shared.clone();
        let db_pool = application.pool.clone();

        let notifier = application.started_notify.clone();
        tokio::spawn(async move {
            application.run_forever().await.unwrap();
        });

        notifier.notified().await;
        let rpc_url = format!(
            "{}://localhost:{}",
            server_config.main_cfg.protocol_http(),
            port
        );
        let mut connected_channel = Endpoint::from_shared(rpc_url.clone())?;
        if server_config.main_cfg.tls.is_tls_on()? {
            let client_cert = std::fs::read_to_string(
                server_config
                    .main_cfg
                    .tls
                    .client_tls_cert_path
                    .clone()
                    .unwrap(),
            )?;
            let client_key = std::fs::read_to_string(
                server_config
                    .main_cfg
                    .tls
                    .client_key_cert_path
                    .clone()
                    .unwrap(),
            )?;
            let client_identity = Identity::from_pem(client_cert.clone(), client_key);
            let server_ca_cert = std::fs::read_to_string(
                server_config.main_cfg.tls.ca_tls_cert_path.clone().unwrap(),
            )?;
            let server_root_ca = Certificate::from_pem(server_ca_cert);
            let tls = ClientTlsConfig::new()
                .ca_certificate(server_root_ca)
                .identity(client_identity);
            connected_channel = Endpoint::from_shared(rpc_url.clone())?.tls_config(tls)?;
        }
        let connected_channel = connected_channel.connect().await?;
        let obj = TestApp {
            port,
            db_url,
            server_drop_handle: Some(abort_handle),
            has_dropped: false,
            app_shared: Some(shared),
            owned_users: vec![],
            db_pool: Some(db_pool),
            rpc_url: rpc_url.clone(),
            clients: Clients {
                auth: AuthServiceClient::new(connected_channel.clone()),
                basic: BasicServiceClient::new(connected_channel),
            },
            should_drop_db: true,
            app_config: server_config,
            rmq_vhost: vhost,
            should_drop_vhost: true,
        };

        Ok(obj)
    }

    pub async fn new_with_existing_instance(cfg: Cfg) -> anyhow::Result<Self> {
        let remote_url = format!(
            "{}://{}:{}",
            cfg.main_cfg.protocol_http(),
            cfg.main_cfg.ip,
            cfg.main_cfg.port
        );
        let vhost = cfg.rabbitmq_cfg.vhost.clone();
        Ok(Self {
            should_drop_db: false,
            port: cfg.main_cfg.port,
            db_url: cfg.db_cfg.url(),
            app_shared: None,
            db_pool: None,
            owned_users: vec![],
            server_drop_handle: None,
            has_dropped: false,
            rpc_url: remote_url.clone(),
            clients: Clients {
                auth: AuthServiceClient::connect(remote_url.clone()).await?,
                basic: BasicServiceClient::connect(remote_url.clone()).await?,
            },
            app_config: cfg,
            rmq_vhost: vhost,
            should_drop_vhost: false,
        })
    }

    pub async fn async_drop(&mut self) {
        tracing::info!("async_drop called");
        for i in &self.owned_users {
            i.lock().await.async_drop().await;
        }
        if let Some(mut handle) = self.server_drop_handle.take() {
            handle.shutdown_all_tasks().await.unwrap();
            tracing::info!("shutdown message sent");
        }

        tracing::info!("app shut down");
        tokio::time::sleep(Duration::from_secs(1)).await;
        if self.should_drop_db {
            match sqlx::Postgres::drop_database(&self.db_url).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("failed to drop the database: {}", e);
                }
            }
        }
        tracing::info!("db deleted");
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
        let user = Arc::new(tokio::sync::Mutex::new(TestUser::random(self).await));
        if self.app_config.main_cfg.tls.is_tls_on()? {
            user.lock().await.tls = self.app_config.main_cfg.tls.clone();
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
    ) -> anyhow::Result<(Vec<TestUserShared>, TestSession)> {
        let mut users = Vec::with_capacity(n);
        for _ in 0..n {
            users.push(self.new_user().await?);
        }
        // create a group in database level
        let session_id = helper::generate_session_id()?;
        // then will join to session and add size column
        process::db::create_session_db(
            session_id,
            0,
            name.into(),
            &self.db_pool.as_ref().unwrap().db_pool,
        )
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
            .clients
            .basic
            .timestamp(TimestampRequest {})
            .await
            .unwrap()
            .into_inner()
            .timestamp
            .unwrap();
        from_google_timestamp(&ret).unwrap()
    }

    pub async fn get_id(&mut self, ocid: OCID) -> Result<ID, tonic::Status> {
        let id: ID = self
            .clients
            .basic
            .get_id(GetIdRequest { ocid: ocid.0 })
            .await?
            .into_inner()
            .id
            .into();
        Ok(id)
    }

    /// Get the database connection.
    ///
    /// # Panics
    ///
    /// Panics if launching with an existing instance.
    pub fn get_db_connection(&self) -> &sea_orm::DatabaseConnection {
        &self.db_pool.as_ref().unwrap().db_pool
    }

    pub async fn check_ban_status(
        &self,
        user: ID,
        session_id: SessionID,
    ) -> anyhow::Result<Option<BanStatus>> {
        let mut redis_connection = self.db_pool.as_ref().unwrap().redis_pool.get().await?;
        Ok(user_banned_status(user, session_id, &mut redis_connection).await?)
    }

    pub async fn check_mute_status(
        &self,
        user: ID,
        session_id: SessionID,
    ) -> anyhow::Result<Option<MuteStatus>> {
        let mut redis_connection = self.db_pool.as_ref().unwrap().redis_pool.get().await?;
        Ok(user_muted_status(user, session_id, &mut redis_connection).await?)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this app");
        }
    }
}

type ConfigWithArgs = (Cfg, ArgsParser);
