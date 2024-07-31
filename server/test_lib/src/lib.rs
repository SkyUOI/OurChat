//! 辅助服务器测试

pub mod login;
pub mod register;

use assert_cmd::prelude::*;
use serde::{Deserialize, Serialize};
use server::consts::RequestType;
use std::{
    fs,
    process::{Child, Command},
    sync::OnceLock,
};
use tokio::net::TcpStream;
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
            .args(&["--cfg", &config_file, "--test-mode"])
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

#[derive(Debug, Serialize, Deserialize)]
struct AccountDeletion {
    code: RequestType,
}

impl AccountDeletion {
    fn new() -> Self {
        Self {
            code: RequestType::Unregister,
        }
    }
}

/// 清理测试环境时顺便测试帐号删除，删除需要在所有测试后运行，所以只能在这里测试
fn test_account_deletion() {}

/// 用于清理测试环境
struct TearDown;

impl Drop for TearDown {
    fn drop(&mut self) {
        test_account_deletion();
    }
}

fn set_up_server() {
    static TMP: OnceLock<SetUpHandle> = OnceLock::new();
    TMP.get_or_init(|| SetUpHandle::new());
}

struct Conn {
    pub ws: ClientWS,
}

impl Drop for Conn {
    fn drop(&mut self) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime
            .block_on(async {
                let ret = self.ws.close(None).await;
                eprintln!("close ws");
                ret
            })
            .unwrap()
    }
}

impl Conn {
    pub fn new(ws: ClientWS) -> Self {
        Self { ws }
    }
}

pub fn init_server() -> &'static ClientWS {
    static TMP: OnceLock<(TearDown, Conn)> = OnceLock::new();
    &TMP.get_or_init(|| {
        set_up_server();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let conn = Conn::new(runtime.block_on(async {
            register::test_register().await;
            login::test_login().await
        }));
        (TearDown {}, conn)
    })
    .1
    .ws
}

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

async fn connect_to_server_internal() -> anyhow::Result<ClientWS> {
    let (ws, _) =
        tokio_tungstenite::connect_async(server::utils::ws_bind_addr().to_string()).await?;
    Ok(ws)
}

/// 连接到服务器
pub async fn get_connection() -> &'static ClientWS {
    init_server()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
    pub ocid: String,
}

pub fn get_test_user() -> &'static TestUser {
    static TMP: OnceLock<TestUser> = OnceLock::new();
    TMP.get_or_init(|| {
        serde_json::from_str(
            &fs::read_to_string("config/test_user.json").expect("Cannot read test_user.json"),
        )
        .unwrap()
    })
}
