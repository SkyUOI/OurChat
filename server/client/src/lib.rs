//! Helper functions for tests

use base::time::TimeStampUtc;
use fake::Fake;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::en;
use fake::faker::name::raw::Name;
use fake::locales::EN;
use itertools::Itertools;
use parking_lot::Mutex;
use rand::Rng;
use server::component::MockEmailSender;
use server::consts::{ID, SessionID};
use server::db::DbCfgTrait;
use server::pb::auth::authorize::v1::auth_request;
use server::pb::auth::register::v1::RegisterRequest;
use server::pb::basic::v1::TimestampRequest;
use server::pb::{
    auth::authorize::v1::AuthRequest,
    auth::v1::auth_service_client::AuthServiceClient,
    basic::v1::basic_service_client::BasicServiceClient,
    ourchat::v1::our_chat_service_client::OurChatServiceClient,
    ourchat::{
        download::v1::DownloadRequest, unregister::v1::UnregisterRequest, upload::v1::UploadRequest,
    },
};
use server::utils::{self, from_google_timestamp, get_available_port};
use server::{Application, ArgsParser, DbPool, ParserCfg, SharedData, ShutdownSdr, process};
use sqlx::migrate::MigrateDatabase;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::metadata::MetadataValue;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, Uri};

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

pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
    pub ocid: String,
    pub id: ID,
    pub port: u16,
    pub token: String,
    pub client: Clients,
    pub rpc_url: String,
    pub oc_server: Option<OCClient>,

    has_dropped: bool,
}

struct FakeManager {
    dup_name: HashSet<String>,
    dup_email: HashSet<String>,
    name_faker: Name<EN>,
    email_faker: FreeEmail<EN>,
}

impl FakeManager {
    fn new() -> Self {
        Self {
            dup_name: HashSet::new(),
            name_faker: en::Name(),
            dup_email: HashSet::new(),
            email_faker: fake::faker::internet::en::FreeEmail(),
        }
    }

    fn generate_unique_name(&mut self) -> String {
        loop {
            let name: String = self.name_faker.fake();
            if !self.dup_name.contains(&name) {
                self.dup_name.insert(name.clone());
                return name;
            }
        }
    }

    fn generate_unique_email(&mut self) -> String {
        loop {
            let email: String = self.email_faker.fake();
            if !self.dup_email.contains(&email) {
                self.dup_email.insert(email.clone());
                return email;
            }
        }
    }
}

static FAKE_MANAGER: LazyLock<Mutex<FakeManager>> =
    LazyLock::new(|| Mutex::new(FakeManager::new()));

// Utils functions implemented
impl TestUser {
    pub async fn random(app: &TestApp) -> Self {
        let name = FAKE_MANAGER.lock().generate_unique_name();
        let email = FAKE_MANAGER.lock().generate_unique_email();
        let url = app.rpc_url.clone();
        Self {
            name,
            password: rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
            email,
            port: app.port,
            has_dropped: false,
            client: app.clients.clone(),
            rpc_url: url,
            // reserved
            ocid: String::default(),
            token: String::default(),
            oc_server: None,
            id: ID::default(),
        }
    }

    pub async fn register_internal(user: &mut TestUser) -> anyhow::Result<()> {
        let request = RegisterRequest {
            name: user.name.clone(),
            password: user.password.clone(),
            email: user.email.clone(),
        };
        let ret = user
            .client
            .auth
            .register(request)
            .await
            .unwrap()
            .into_inner();
        user.ocid = ret.ocid;
        user.id = ID(ret.id);
        user.token = ret.token;
        let chann = Channel::builder(Uri::from_maybe_shared(user.rpc_url.clone()).unwrap())
            .connect()
            .await
            .unwrap();
        let token: MetadataValue<_> = user.token.to_string().parse().unwrap();
        user.oc_server = Some(OurChatServiceClient::with_interceptor(
            chann,
            Box::new(move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert("token", token.clone());
                Ok(req)
            }),
        ));
        Ok(())
    }

    async fn async_drop(&mut self) {
        claims::assert_ok!(self.unregister().await);
        tracing::info!("unregister done");
        self.has_dropped = true;
    }
}

// Features implemented
impl TestUser {
    pub async fn accept_session(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn register(&mut self) -> anyhow::Result<()> {
        Self::register_internal(self).await
    }

    pub async fn unregister(&mut self) -> anyhow::Result<()> {
        let req = UnregisterRequest {};
        self.oc().unregister(req).await.unwrap();
        Ok(())
    }

    pub fn oc(&mut self) -> &mut OCClient {
        self.oc_server.as_mut().unwrap()
    }

    pub async fn ocid_auth(&mut self) -> anyhow::Result<()> {
        let login_req = AuthRequest {
            account: Some(auth_request::Account::Ocid(self.ocid.clone())),
            password: self.password.clone(),
        };
        let ret = self.client.auth.auth(login_req).await.unwrap().into_inner();
        self.token = ret.token.clone();
        Ok(())
    }

    pub async fn email_auth(&mut self) -> anyhow::Result<()> {
        self.email_auth_internal(self.password.clone()).await
    }

    pub async fn email_auth_internal(&mut self, password: impl Into<String>) -> anyhow::Result<()> {
        let login_req = AuthRequest {
            account: Some(auth_request::Account::Email(self.email.clone())),
            password: password.into(),
        };
        let ret = self.client.auth.auth(login_req).await?.into_inner();
        assert_eq!(*self.id, ret.id);
        Ok(())
    }

    pub async fn post_file(&mut self, content: String) -> anyhow::Result<String> {
        let size = content.len();
        let hash = base::sha3_256(content.as_bytes());
        // post file
        let mut files_content = vec![UploadRequest::new_header(size, hash, false)];
        let content = content.into_bytes();
        for i in &content.into_iter().chunks(1024 * 1024) {
            files_content.push(UploadRequest::new_content(i.collect()));
        }
        let ret = self.oc().upload(tokio_stream::iter(files_content)).await?;
        let ret = ret.into_inner();
        let key = ret.key;
        Ok(key)
    }

    pub async fn download_file(&mut self, key: impl Into<String>) -> anyhow::Result<Vec<u8>> {
        let files_part = self
            .oc()
            .download(DownloadRequest { key: key.into() })
            .await?;
        // Allow
        let mut files_part = files_part.into_inner();
        let mut file_download = Vec::new();
        while let Some(part) = files_part.next().await {
            let part = part?;
            file_download.extend_from_slice(&part.data);
        }
        Ok(file_download)
    }
}

impl Drop for TestUser {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this user");
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clients {
    pub auth: AuthServiceClient<Channel>,
    pub basic: BasicServiceClient<Channel>,
}

/// A test client
///
/// # Details
/// Some members are Option because you can new a TestApp by connecting to an existing server or not
#[derive(Clone)]
pub struct TestApp {
    pub port: u16,
    pub http_port: u16,
    pub db_url: String,
    pub app_shared: Option<Arc<SharedData<MockEmailSender>>>,
    pub db_pool: Option<DbPool>,
    pub http_client: reqwest::Client,
    pub owned_users: Vec<Arc<tokio::sync::Mutex<TestUser>>>,
    pub clients: Clients,
    pub rpc_url: String,
    pub app_config: server::Cfg,

    has_dropped: bool,
    server_drop_handle: Option<ShutdownSdr>,
    should_drop_db: bool,
}

trait TestAppTrait {
    fn test() -> Self;
}

impl TestAppTrait for ArgsParser {
    fn test() -> Self {
        Self {
            port: Some(get_available_port()),
            http_port: Some(0),
            enable_cmd: Some(false),
            shared_cfg: ParserCfg {
                test_mode: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub type TestUserShared = Arc<tokio::sync::Mutex<TestUser>>;

pub struct TestSession {
    pub session_id: SessionID,
}

impl TestSession {
    pub fn new(session_id: SessionID) -> Self {
        Self { session_id }
    }
}

impl TestApp {
    pub async fn new_with_launching_instance(
        email_client: Option<MockEmailSender>,
    ) -> anyhow::Result<Self> {
        let args = ArgsParser::test();
        let mut server_config = server::get_configuration(args.shared_cfg.config.as_ref())?;
        // should create different database for each test
        let db = uuid::Uuid::new_v4().to_string();
        server_config.db_cfg.db = db;
        let db_url = server_config.db_cfg.url();
        let mut application = Application::build(args, server_config.clone(), email_client).await?;
        let port = application.get_port();
        let http_port = application.get_http_port();
        let abort_handle = application.get_abort_handle();
        let shared = application.shared.clone();
        let db_pool = application.pool.clone();

        let notifier = application.started_notify.clone();
        tokio::spawn(async move {
            application.run_forever().await.unwrap();
        });
        notifier.notified().await;
        let rpc_url = format!("http://localhost:{}", port);
        let obj = TestApp {
            port,
            http_port,
            db_url,
            server_drop_handle: Some(abort_handle),
            has_dropped: false,
            app_shared: Some(shared),
            owned_users: vec![],
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
            db_pool: Some(db_pool),
            rpc_url: rpc_url.clone(),
            clients: Clients {
                auth: AuthServiceClient::connect(rpc_url.clone()).await?,
                basic: BasicServiceClient::connect(rpc_url.clone()).await?,
            },
            should_drop_db: true,
            app_config: server_config,
        };
        Ok(obj)
    }

    pub async fn new_with_existing_instance(cfg: server::Cfg) -> anyhow::Result<Self> {
        let remote_url = format!(
            "{}://{}:{}",
            cfg.main_cfg.protocol_http(),
            cfg.main_cfg.ip,
            cfg.main_cfg.port
        );
        Ok(Self {
            should_drop_db: false,
            port: cfg.main_cfg.port,
            http_port: cfg.main_cfg.http_port,
            db_url: cfg.db_cfg.url(),
            app_shared: None,
            db_pool: None,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
            owned_users: vec![],
            server_drop_handle: None,
            has_dropped: false,
            rpc_url: remote_url.clone(),
            clients: Clients {
                auth: AuthServiceClient::connect(remote_url.clone()).await?,
                basic: BasicServiceClient::connect(remote_url.clone()).await?,
            },
            app_config: cfg,
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

        tracing::info!("app shutdowned");
        tokio::time::sleep(Duration::from_secs(1)).await;
        if self.should_drop_db {
            match sqlx::Postgres::drop_database(&self.db_url).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("failed to drop database: {}", e);
                }
            }
        }
        tracing::info!("db deleted");
        self.has_dropped = true;
    }

    pub async fn verify(&mut self, token: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.http_get(format!("verify/confirm?token={}", token))
            .await
    }

    pub async fn new_user(&mut self) -> anyhow::Result<TestUserShared> {
        let user = Arc::new(tokio::sync::Mutex::new(TestUser::random(self).await));
        user.lock().await.register().await?;
        self.owned_users.push(user.clone());
        Ok(user)
    }

    pub async fn http_get(
        &self,
        name: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(format!(
                "{}://{}:{}/v1/{}",
                self.app_config.main_cfg.protocol_http(),
                self.app_config.main_cfg.ip,
                self.http_port,
                name.as_ref()
            ))
            .send()
            .await
    }

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
        let session_id = utils::generate_session_id()?;
        process::db::create_session_db(
            session_id,
            n,
            name.into(),
            &self.db_pool.as_ref().unwrap().db_pool,
        )
        .await?;
        tracing::info!("create session:{}", session_id);
        let mut id_vec = vec![];
        for i in &users {
            let id =
                process::db::get_id(&i.lock().await.ocid, self.db_pool.as_ref().unwrap()).await?;
            id_vec.push(id);
        }
        tracing::debug!("id:{:?}", id_vec);
        process::db::batch_add_to_session(
            &self.db_pool.as_ref().unwrap().db_pool,
            session_id,
            &id_vec,
        )
        .await?;
        Ok((users, TestSession::new(session_id)))
    }

    pub async fn new_session(
        &mut self,
        n: usize,
        name: impl Into<String>,
    ) -> anyhow::Result<(Vec<TestUserShared>, TestSession)> {
        todo!()
    }

    /// # Warning
    /// Must request the server, shouldn't build a time from local by chrono, because some tests
    /// rely on this behaviour
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
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this app");
        }
    }
}
