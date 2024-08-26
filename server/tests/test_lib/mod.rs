//! 辅助服务器测试

pub mod login;
pub mod register;
pub mod unregister;

use assert_cmd::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    process::{Child, Command},
    ptr::drop_in_place,
    sync::{Arc, LazyLock},
};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

#[macro_export]
macro_rules! cleanup {
    () => {
        #[test]
        #[serial_test::serial]
        fn cleanup() {
            test_lib::teardown();
        }
    };
}

struct SetUpHandle {
    server_handle: Child,
}

impl SetUpHandle {
    fn new() -> Self {
        // Audit that the environment access only happens in single-threaded code.
        unsafe { std::env::set_var("RUST_LOG", "DEBUG") };
        let config_file = match std::env::var("OURCHAT_CONFIG_FILE") {
            Ok(v) => v,
            Err(_) => "../config/mysql/ourchat.toml".to_string(),
        };
        let mut argv_list: Vec<&str> = vec!["--cfg", &config_file, "--test-mode"];
        let argv = match std::env::var("OURCHAT_ARGVS") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        };
        if !argv.is_empty() {
            argv_list.push(&argv);
        }
        let cmd = Command::cargo_bin("server")
            .unwrap()
            .args(&argv_list)
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

struct UnregisterHook;

static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

impl Drop for UnregisterHook {
    fn drop(&mut self) {
        // 注意此处不可使用 get_tokio_runtime()，因为此时get_tokio_runtime可能已经被析构
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(unregister::test_unregister());
    }
}

static SERVER_HANDLE: LazyLock<SetUpHandle> = LazyLock::new(SetUpHandle::new);

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
    INIT_SERVER.1.ws.clone()
}

static INIT_SERVER: LazyLock<(UnregisterHook, Conn)> = LazyLock::new(|| {
    let _ = &*SERVER_HANDLE;
    let conn = Conn::new(Arc::new(Mutex::new(TOKIO_RUNTIME.block_on(async {
        let ocid = register::test_register().await;
        login::test_login(ocid).await
    }))));
    (UnregisterHook {}, conn)
});

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

/// 创建到服务器的连接
async fn create_connection() -> anyhow::Result<ClientWS> {
    let (ws, _) = tokio_tungstenite::connect_async(server::utils::WS_BIND_ADDR.to_string()).await?;
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
}

/// 测试用户
static TEST_USER: LazyLock<TestUser> = LazyLock::new(|| {
    serde_json::from_str(
        &fs::read_to_string("config/test_user.json").expect("Cannot read test_user.json"),
    )
    .unwrap()
});

pub fn teardown() {
    let ptr = (&*INIT_SERVER) as *const (UnregisterHook, Conn) as *mut (UnregisterHook, Conn);
    unsafe { drop_in_place(ptr) }
    let server_handle = &*SERVER_HANDLE as *const SetUpHandle as *mut SetUpHandle;
    unsafe { drop_in_place(server_handle) }
}
