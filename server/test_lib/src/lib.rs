//! 辅助服务器测试

pub mod login;
pub mod register;
pub mod unregister;

use assert_cmd::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    process::{Child, Command},
    sync::{Arc, OnceLock},
};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

struct SetUpHandle {
    server_handle: Child,
}

impl SetUpHandle {
    fn new() -> Self {
        std::env::set_var("RUST_LOG", "DEBUG");
        let config_file = match std::env::var("OURCHAT_CONFIG_FILE") {
            Ok(v) => v,
            Err(_) => "../config/ourchat.toml".to_string(),
        };
        let cmd = Command::cargo_bin("server")
            .unwrap()
            .args(["--cfg", &config_file, "--test-mode"])
            .spawn()
            .unwrap();
        Self { server_handle: cmd }
    }
}

impl Drop for SetUpHandle {
    fn drop(&mut self) {
        self.server_handle.kill().unwrap();
    }
}

/// 用于清理测试环境
struct TearDown;

fn get_tokio_runtime() -> &'static tokio::runtime::Runtime {
    static TMP: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    TMP.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

impl Drop for TearDown {
    fn drop(&mut self) {
        // 注意此处不可使用 get_tokio_runtime()，因为此时get_tokio_runtime可能已经被析构
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(unregister::test_unregister());
    }
}

fn set_up_server() {
    static TMP: OnceLock<SetUpHandle> = OnceLock::new();
    TMP.get_or_init(SetUpHandle::new);
}

struct Conn {
    pub ws: Arc<Mutex<ClientWS>>,
}

impl Drop for Conn {
    fn drop(&mut self) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime
            .block_on(async {
                let ret = self.ws.lock().await.close(None).await;
                eprintln!("close ws");
                ret
            })
            .unwrap()
    }
}

impl Conn {
    pub fn new(ws: Arc<Mutex<ClientWS>>) -> Self {
        Self { ws }
    }
}

/// 初始化服务器并创建到服务器的连接
pub fn init_server() -> Arc<Mutex<ClientWS>> {
    static TMP: OnceLock<(TearDown, Conn)> = OnceLock::new();
    TMP.get_or_init(|| {
        set_up_server();
        let conn = Conn::new(Arc::new(Mutex::new(get_tokio_runtime().block_on(async {
            register::test_register().await;
            login::test_login().await
        }))));
        (TearDown {}, conn)
    })
    .1
    .ws
    .clone()
}

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

/// 创建到服务器的连接
async fn create_connection() -> anyhow::Result<ClientWS> {
    let (ws, _) =
        tokio_tungstenite::connect_async(server::utils::ws_bind_addr().to_string()).await?;
    Ok(ws)
}

/// 连接到服务器,该方法可以反复调用
pub fn get_connection() -> Arc<Mutex<ClientWS>> {
    init_server()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
    pub ocid: String,
}

/// 获取测试用户
pub fn get_test_user() -> &'static TestUser {
    static TMP: OnceLock<TestUser> = OnceLock::new();
    TMP.get_or_init(|| {
        serde_json::from_str(
            &fs::read_to_string("config/test_user.json").expect("Cannot read test_user.json"),
        )
        .unwrap()
    })
}
