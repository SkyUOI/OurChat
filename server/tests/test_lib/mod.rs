//! 辅助服务器测试

pub mod login;
pub mod register;
pub mod unregister;

use rand::Rng;
use serde::{Deserialize, Serialize};
use server::consts::DEFAULT_PORT;
use server::utils::gen_ws_bind_addr;
use std::sync::{Arc, LazyLock, OnceLock};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

struct UnregisterHook;

impl UnregisterHook {
    pub async fn delete(&self) {
        unregister::test_unregister().await
    }
}

struct Conn {
    pub ws: Arc<Mutex<ClientWS>>,
}

impl Conn {
    pub fn new(ws: Arc<Mutex<ClientWS>>) -> Self {
        Self { ws }
    }

    pub async fn delete(&self) {
        let ret = self.ws.lock().await.close(None).await;
        eprintln!("close ws");
        ret.unwrap()
    }
}

async fn init_server() -> &'static (UnregisterHook, Conn) {
    static LOCK: OnceLock<(UnregisterHook, Conn)> = OnceLock::new();
    match LOCK.get() {
        Some(data) => data,
        None => {
            let ocid = register::test_register().await;
            let conn = login::test_login(ocid).await;
            let conn = Conn::new(Arc::new(Mutex::new(conn)));
            let data = (UnregisterHook {}, conn);
            LOCK.get_or_init(|| data)
        }
    }
}

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

static WS_CLIENT_BIND_ADDR: LazyLock<String> =
    LazyLock::new(|| gen_ws_bind_addr("127.0.0.1", DEFAULT_PORT));

/// 创建到服务器的连接
async fn create_connection() -> anyhow::Result<ClientWS> {
    let (ws, _) = tokio_tungstenite::connect_async(WS_CLIENT_BIND_ADDR.clone()).await?;
    Ok(ws)
}

/// 连接到服务器,该方法可以反复调用
pub async fn get_connection() -> Arc<Mutex<ClientWS>> {
    init_server().await.1.ws.clone()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
}

impl TestUser {
    pub fn random() -> Self {
        Self {
            name: rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(15)
                .map(char::from)
                .collect(),
            password: rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
            email: format!(
                "{}@test.com",
                rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(30)
                    .map(char::from)
                    .collect::<String>()
            ),
        }
    }
}

/// 测试用户
static TEST_USER: LazyLock<TestUser> = LazyLock::new(TestUser::random);

pub async fn teardown() {
    init_server().await.0.delete().await;
    init_server().await.1.delete().await;
    println!("teardown done");
}

#[macro_export]
macro_rules! register_test {
    ($($testname:ident),+) => {
        #[tokio::test]
        async fn test_internal() {
            $(
                println!("Testing {}...", stringify!($testname));
                $testname().await;
            )+
            $crate::test_lib::teardown().await;
        }
    }
}
