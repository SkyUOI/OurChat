//! Helper functions for tests

use fake::Fake;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::en;
use fake::faker::name::raw::Name;
use fake::locales::EN;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use server::connection::client_response::{self, UnregisterResponse};
use server::consts::MessageType;
use server::requests::{self, Login, LoginType, Register, Unregister};
use server::utils::gen_ws_bind_addr;
use server::{Application, ArgsParser, ParserCfg};
use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;
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
    pub async fn new() -> anyhow::Result<Self> {
        let args = server::ArgsParser::test();
        let mut application = Application::build(args, None).await?;
        let port = application.get_port();
        let http_port = application.get_http_port();
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
        };
        obj.register().await;
        Ok(obj)
    }

    pub async fn new_logined() -> anyhow::Result<Self> {
        let mut obj = Self::new().await?;
        obj.email_login().await;
        Ok(obj)
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

    pub async fn register(&mut self) {
        let request = Register::new(
            self.user.name.clone(),
            self.user.password.clone(),
            self.user.email.clone(),
        );
        self.connection
            .send(Message::Text(serde_json::to_string(&request).unwrap()))
            .await
            .unwrap();
        let ret = self.connection.next().await.unwrap().unwrap();
        self.connection.close(None).await.unwrap();
        let json: client_response::RegisterResponse =
            serde_json::from_str(&ret.to_string()).unwrap();
        assert_eq!(json.status, requests::Status::Success);
        assert_eq!(json.code, MessageType::RegisterRes);
        self.user.ocid = json.ocid.unwrap();
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

    pub async fn ocid_login(&mut self) {
        self.establish_connection().await.unwrap();
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
        let json: client_response::LoginResponse =
            serde_json::from_str(ret.to_text().unwrap()).unwrap();
        assert_eq!(json.code, MessageType::LoginRes);
    }

    pub async fn email_login(&mut self) {
        self.establish_connection().await.unwrap();
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
        let json: client_response::LoginResponse =
            serde_json::from_str(ret.to_text().unwrap()).unwrap();
        assert_eq!(json.code, MessageType::LoginRes);
        assert_eq!(json.ocid.unwrap(), self.user.ocid);
    }

    pub async fn async_drop(&mut self) {
        self.unregister().await;
        self.close_connection().await;
    }

    pub async fn close_connection(&mut self) {
        self.connection.close(None).await.unwrap();
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
