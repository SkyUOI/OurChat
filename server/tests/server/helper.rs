//! Helper functions for tests

use fake::Fake;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::en;
use fake::faker::name::raw::Name;
use fake::locales::EN;
use futures_util::stream::FusedStream;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use server::client::requests::{self, Login, LoginType, Register, Unregister};
use server::client::response::{self, UnregisterResponse};
use server::component::MockEmailSender;
use server::consts::MessageType;
use server::db::{DbCfg, DbCfgTrait, DbType};
use server::utils::gen_ws_bind_addr;
use server::{Application, ArgsParser, ParserCfg, SharedData, ShutdownSdr};
use sqlx::migrate::MigrateDatabase;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::fs::remove_file;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
    pub ocid: String,
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

impl TestUser {
    pub fn random() -> Self {
        let name = FAKE_MANAGER.lock().generate_unique_name();
        let email = FAKE_MANAGER.lock().generate_unique_email();
        Self {
            name,
            password: rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
            email,
            // reserved
            ocid: String::default(),
        }
    }
}

pub struct TestApp {
    pub port: u16,
    pub http_port: u16,
    pub connection: ClientWS,
    pub user: TestUser,
    pub handle: JoinHandle<()>,
    pub db_url: String,
    pub app_shared: Arc<SharedData<MockEmailSender>>,
    pub http_client: reqwest::Client,

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

impl TestApp {
    pub async fn new(email_client: Option<MockEmailSender>) -> anyhow::Result<Self> {
        let args = ArgsParser::test();
        let server_config = server::get_configuration(args.shared_cfg.config.as_ref())?;
        let mut server_config = server::Cfg::new(server_config)?;
        // should create different database for each test
        let db_url;
        let db = uuid::Uuid::new_v4().to_string();
        match &mut server_config.db_cfg {
            DbCfg::Mysql(cfg) => {
                cfg.db = db;
                db_url = cfg.url();
            }
            DbCfg::Sqlite(cfg) => {
                cfg.path = PathBuf::from(format!(".{}.db", db));
                db_url = cfg.url();
            }
        }
        let mut application = Application::build(args, server_config.clone(), email_client).await?;
        let port = application.get_port();
        let http_port = application.get_http_port();
        let abort_handle = application.get_abort_handle();
        let shared = application.shared.clone();

        let handle = tokio::spawn(async move {
            application.run_forever().await.unwrap();
        });

        let connection = Self::establish_connection_internal(port).await?;

        let mut obj = TestApp {
            port,
            http_port,
            connection,
            user: TestUser::random(),
            handle,
            db_url,
            server_config,
            has_dropped: false,
            server_drop_handle: abort_handle,
            app_shared: shared,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
        };
        println!("register user: {:?}", obj.user);
        obj.register().await;
        Ok(obj)
    }

    pub async fn new_logined(email_client: Option<MockEmailSender>) -> anyhow::Result<Self> {
        let mut obj = Self::new(email_client).await?;
        obj.email_login().await?;
        Ok(obj)
    }

    pub async fn send(&mut self, msg: Message) -> anyhow::Result<()> {
        self.connection.send(msg).await?;
        Ok(())
    }

    pub async fn get(&mut self) -> anyhow::Result<Message> {
        Ok(self.connection.next().await.unwrap().unwrap())
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

    async fn establish_connection(&mut self) -> anyhow::Result<()> {
        self.connection.close(None).await.ok();
        let conn = Self::establish_connection_internal(self.port).await?;
        self.connection = conn;
        Ok(())
    }

    pub async fn register_internal(user: &mut TestUser, conn: &mut ClientWS) -> anyhow::Result<()> {
        let request = Register::new(user.name.clone(), user.password.clone(), user.email.clone());
        conn.send(Message::Text(serde_json::to_string(&request).unwrap()))
            .await
            .unwrap();
        let ret = conn.next().await.unwrap().unwrap();
        let json: response::RegisterResponse = serde_json::from_str(&ret.to_string()).unwrap();
        assert_eq!(json.status, requests::Status::Success);
        assert_eq!(json.code, MessageType::RegisterRes);
        user.ocid = json.ocid.unwrap();
        Ok(())
    }

    pub async fn register(&mut self) {
        Self::register_internal(&mut self.user, &mut self.connection)
            .await
            .unwrap();
        self.establish_connection().await.unwrap();
    }

    pub async fn unregister(&mut self) {
        let req = Unregister::new();
        self.connection
            .send(Message::text(serde_json::to_string(&req).unwrap()))
            .await
            .unwrap();
        let ret = self.connection.next().await.unwrap().unwrap();
        let json: UnregisterResponse = serde_json::from_str(ret.to_text().unwrap()).unwrap();
        assert_eq!(json.code, MessageType::UnregisterRes);
        assert_eq!(json.status, requests::Status::Success);
    }

    pub async fn ocid_login(&mut self) -> anyhow::Result<()> {
        let login_req = Login::new(
            self.user.ocid.clone(),
            self.user.password.clone(),
            LoginType::Ocid,
        );
        self.connection
            .send(Message::Text(serde_json::to_string(&login_req).unwrap()))
            .await
            .unwrap();
        let ret = self.connection.next().await.unwrap().unwrap();
        let json: response::LoginResponse = serde_json::from_str(ret.to_text().unwrap()).unwrap();
        if json.code != MessageType::LoginRes {
            anyhow::bail!("Failed to login,code is not login response");
        }
        Ok(())
    }

    pub async fn email_login(&mut self) -> anyhow::Result<()> {
        let login_req = Login::new(
            self.user.email.clone(),
            self.user.password.clone(),
            LoginType::Email,
        );
        self.connection
            .send(Message::Text(serde_json::to_string(&login_req).unwrap()))
            .await
            .unwrap();
        let ret = self.connection.next().await.unwrap().unwrap();
        let json: response::LoginResponse = serde_json::from_str(ret.to_text().unwrap())?;
        if json.code != MessageType::LoginRes {
            anyhow::bail!("Failed to login,code is not login response");
        }
        if let Some(ocid) = json.ocid.clone() {
            if ocid != self.user.ocid {
                anyhow::bail!("Failed to login,ocid is not correct");
            }
        } else {
            anyhow::bail!("Failed to login,ocid is not found");
        }
        Ok(())
    }

    pub async fn async_drop(&mut self) {
        tracing::info!("async_drop called");
        self.unregister().await;
        tracing::info!("unregister done");
        self.close_connection().await;
        tracing::info!("connection closed");
        self.server_drop_handle.send(()).unwrap();
        tracing::info!("shutdown message sent");
        loop {
            if self.handle.is_finished() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        tracing::info!("app shutdowned");
        match self.server_config.main_cfg.db_type {
            DbType::Sqlite => {
                sqlx::Sqlite::drop_database(&self.db_url).await.unwrap();
                let mut path = PathBuf::from(&self.db_url.strip_prefix("sqlite://").unwrap());
                path.set_extension("db-shm");
                remove_file(&path).await.ok();
                path.set_extension("db-wal");
                remove_file(&path).await.ok();
            }
            DbType::MySql => {
                sqlx::MySql::drop_database(&self.db_url).await.unwrap();
            }
        }
        tracing::info!("db deleted");
        self.has_dropped = true;
    }

    pub async fn close_connection(&mut self) {
        self.connection.close(None).await.unwrap();
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

    pub async fn new_user(&mut self) -> anyhow::Result<(TestUser, ClientWS)> {
        let mut user = TestUser::random();
        let mut conn = Self::establish_connection_internal(self.port).await?;
        Self::register_internal(&mut user, &mut conn).await?;
        Ok((user, conn))
    }

    pub async fn accept_session(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.has_dropped {
            panic!("async_drop is not called to drop this app");
        }
    }
}
