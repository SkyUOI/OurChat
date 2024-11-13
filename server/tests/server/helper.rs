//! Helper functions for tests

use anyhow::Context;
use base::time::TimeStamp;
use fake::Fake;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::en;
use fake::faker::name::raw::Name;
use fake::locales::EN;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use rand::Rng;
use server::client::MsgConvert;
use server::client::requests::{self, LoginRequest, LoginType, RegisterRequest, UnregisterRequest};
use server::client::response::{self};
use server::component::MockEmailSender;
use server::consts::{MessageType, SessionID};
use server::db::DbCfgTrait;
use server::utils::{self, gen_ws_bind_addr};
use server::{Application, ArgsParser, DbPool, ParserCfg, SharedData, ShutdownSdr, connection};
use sqlx::migrate::MigrateDatabase;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
    pub ocid: String,
    pub connection: Option<ClientWS>,
    pub port: u16,

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
        let mut ret = Self {
            name,
            password: rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
            email,
            connection: None,
            port: app.port,
            has_dropped: false,
            // reserved
            ocid: String::default(),
        };
        ret.register().await;
        ret
    }

    pub async fn send(&mut self, msg: Message) -> anyhow::Result<()> {
        self.get_conn().send(msg).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> anyhow::Result<Message> {
        Ok(self.get_conn().next().await.unwrap().unwrap())
    }

    pub async fn get_text(&mut self) -> anyhow::Result<String> {
        Ok(self.get_conn().next().await.unwrap()?.to_text()?.to_owned())
    }

    pub fn get_conn(&mut self) -> &mut ClientWS {
        self.connection.as_mut().unwrap()
    }

    async fn establish_connection(&mut self) -> anyhow::Result<()> {
        if let Some(conn) = &mut self.connection {
            conn.close(None).await.ok();
        }
        let conn = establish_connection_internal(self.port).await?;
        self.connection = Some(conn);
        Ok(())
    }

    pub async fn register_internal(user: &mut TestUser) -> anyhow::Result<()> {
        let request =
            RegisterRequest::new(user.name.clone(), user.password.clone(), user.email.clone());
        let conn = user.get_conn();
        conn.send(request.to_msg()).await.unwrap();
        let ret = conn.next().await.unwrap().unwrap();
        let json: response::RegisterResponse = serde_json::from_str(&ret.to_string()).unwrap();
        assert_eq!(json.status, requests::Status::Success);
        assert_eq!(json.code, MessageType::RegisterRes);
        user.ocid = json.ocid;
        Ok(())
    }

    pub async fn close_connection(&mut self) {
        if let Some(conn) = &mut self.connection {
            conn.close(None).await.unwrap();
        }
    }

    async fn async_drop(&mut self) {
        claims::assert_ok!(self.unregister().await);
        tracing::info!("unregister done");
        self.close_connection().await;
        tracing::info!("connection closed");
        self.has_dropped = true;
    }
}

// Features implemented
impl TestUser {
    pub async fn accept_session(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn register(&mut self) {
        self.establish_connection().await.unwrap();
        Self::register_internal(self).await.unwrap();
    }

    pub async fn unregister(&mut self) -> anyhow::Result<()> {
        let req = UnregisterRequest::new();
        self.get_conn()
            .send(Message::text(serde_json::to_string(&req).unwrap()))
            .await
            .unwrap();
        let ret = self.get_conn().next().await.unwrap()?;
        let json: UnregisterResponse = serde_json::from_str(ret.to_text()?)?;
        assert_eq!(json.code, MessageType::UnregisterRes);
        Ok(())
    }

    pub async fn ocid_login(&mut self) -> anyhow::Result<()> {
        let login_req =
            LoginRequest::new(self.ocid.clone(), self.password.clone(), LoginType::Ocid);
        self.get_conn().send(login_req.to_msg()).await.unwrap();
        let ret = self.get_conn().next().await.unwrap().unwrap();
        let json: response::LoginResponse = serde_json::from_str(ret.to_text().unwrap()).unwrap();
        if json.code != MessageType::LoginRes {
            anyhow::bail!("Failed to login,code is not login response");
        }
        Ok(())
    }

    pub async fn email_login(&mut self) -> anyhow::Result<()> {
        let login_req =
            LoginRequest::new(self.email.clone(), self.password.clone(), LoginType::Email);
        self.get_conn().send(login_req.to_msg()).await.unwrap();
        let ret = self.get_conn().next().await.unwrap().unwrap();
        let json: response::LoginResponse =
            serde_json::from_str(ret.to_text().unwrap()).with_context(|| ret)?;
        if json.code != MessageType::LoginRes {
            anyhow::bail!("Failed to login,code is not login response");
        }
        if let Some(ocid) = json.ocid.clone() {
            if ocid != self.ocid {
                anyhow::bail!("Failed to login,ocid is not correct");
            }
        } else {
            anyhow::bail!("Failed to login,ocid is not found");
        }
        Ok(())
    }
}

impl Drop for TestUser {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this user");
        }
    }
}

async fn establish_connection_internal(port: u16) -> anyhow::Result<ClientWS> {
    let mut connection = None;
    // server maybe not ready
    let ip = gen_ws_bind_addr("127.0.0.1", port);
    for _ in 0..10 {
        match tokio_tungstenite::connect_async(&ip).await {
            Ok((conn, _)) => {
                connection = Some(conn);
                break;
            }
            Err(e) => {
                println!("{} ", e);
            }
        };
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if connection.is_none() {
        anyhow::bail!(format!("Failed to connect to server {}", ip));
    }
    Ok(connection.unwrap())
}

#[derive(Clone)]
pub struct TestApp {
    pub port: u16,
    pub http_port: u16,
    pub db_url: String,
    pub app_shared: Arc<SharedData<MockEmailSender>>,
    pub db_pool: DbPool,
    pub http_client: reqwest::Client,
    owned_users: Vec<Arc<tokio::sync::Mutex<TestUser>>>,

    server_config: server::Cfg,
    has_dropped: bool,
    server_drop_handle: ShutdownSdr,
}

trait TestAppTrait {
    fn test() -> Self;
}

impl TestAppTrait for ArgsParser {
    fn test() -> Self {
        Self {
            port: Some(0),
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
    pub async fn new(email_client: Option<MockEmailSender>) -> anyhow::Result<Self> {
        let args = ArgsParser::test();
        let server_config = server::get_configuration(args.shared_cfg.config.as_ref())?;
        let mut server_config = server::Cfg::new(server_config)?;
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

        tokio::spawn(async move {
            application.run_forever().await.unwrap();
        });

        let obj = TestApp {
            port,
            http_port,
            db_url,
            server_config,
            server_drop_handle: abort_handle,
            has_dropped: false,
            app_shared: shared,
            owned_users: vec![],
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
            db_pool,
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

        tracing::info!("app shutdowned");
        tokio::time::sleep(Duration::from_secs(1)).await;
        match sqlx::Postgres::drop_database(&self.db_url).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("failed to drop database: {}", e);
            }
        }
        tracing::info!("db deleted");
        self.has_dropped = true;
    }

    pub async fn verify(&mut self, token: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(format!(
                "http://127.0.0.1:{}/v1/verify/confirm?token={}",
                self.http_port, token
            ))
            .send()
            .await
    }

    pub async fn new_user(&mut self) -> anyhow::Result<TestUserShared> {
        let user = Arc::new(tokio::sync::Mutex::new(TestUser::random(self).await));
        user.lock().await.close_connection().await;
        user.lock().await.establish_connection().await?;
        self.owned_users.push(user.clone());
        Ok(user)
    }

    pub async fn new_user_logined(&mut self) -> anyhow::Result<TestUserShared> {
        let user = Arc::new(tokio::sync::Mutex::new(TestUser::random(self).await));
        self.owned_users.push(user.clone());
        Ok(user)
    }

    pub async fn http_get(&self, name: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .http_client
            .get(format!("http://127.0.0.1:{}/v1/{}", self.http_port, name))
            .send()
            .await?)
    }

    pub async fn post_file(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn new_session(
        &mut self,
        n: usize,
        name: impl Into<String>,
    ) -> anyhow::Result<(Vec<TestUserShared>, TestSession)> {
        let mut users = Vec::with_capacity(n);
        for _ in 0..n {
            users.push(self.new_user_logined().await?);
        }
        // create a group in database level
        let session_id = utils::generate_session_id()?;
        connection::db::create_session(session_id, n, name.into(), &self.db_pool.db_pool)
            .await?
            .unwrap();
        tracing::info!("create session:{}", session_id);
        let mut id_vec = vec![];
        for i in &users {
            let id = connection::db::get_id(&i.lock().await.ocid, &self.db_pool).await?;
            id_vec.push(id);
        }
        tracing::debug!("id:{:?}", id_vec);
        server::connection::db::batch_add_to_session(&self.db_pool.db_pool, session_id, &id_vec)
            .await?;
        Ok((users, TestSession::new(session_id)))
    }

    /// # Warning
    /// Must request the server, shouldn't build a time from local by chrono, because some tests
    /// rely on this behaviour
    pub async fn get_timestamp(&self) -> TimeStamp {
        let response = self
            .http_get("timestamp")
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        let text = response.text().await.unwrap();
        chrono::DateTime::parse_from_rfc3339(&text).unwrap()
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this app");
        }
    }
}
