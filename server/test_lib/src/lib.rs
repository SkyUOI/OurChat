//! 辅助服务器测试

use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use server::{consts::RequestType, requests::LoginType};
use std::{fs, sync::OnceLock, thread};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

fn set_up_server() {
    let mut cmd = assert_cmd::Command::cargo_bin("server").unwrap();
    let ret = cmd
        .arg("--cfg")
        .arg("../config/ourchat.toml")
        .arg("--test-mode")
        .assert();
    eprintln!("Server Error:{}", ret);
}

#[derive(Debug, Serialize, Deserialize)]
struct Register {
    code: RequestType,
    name: String,
    password: String,
    email: String,
}

impl Register {
    fn new(name: String, password: String, email: String) -> Self {
        Self {
            code: RequestType::Register,
            name,
            password,
            email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountDeletion {
    code: RequestType,
}

impl AccountDeletion {
    fn new() -> Self {
        Self {
            code: RequestType::AccountDeletion,
        }
    }
}

/// 在这里测试注册顺便初始化服务器，注册需要在所有测试前运行，所以只能在这里测试
async fn test_register() {
    let user = get_test_user();
    let request = Register::new(user.name.clone(), user.password.clone(), user.email.clone());
    let mut stream = None;
    // 服务器启动可能没那么快
    for i in 0..10 {
        eprintln!("Try to connect to server:{}", i);
        let ret = connect_to_server_internal().await;
        if ret.is_ok() {
            stream = ret.ok();
            break;
        }
        if i == 9 {
            panic!("Cannot connect to server");
        }
        thread::sleep(std::time::Duration::from_millis(1000));
    }
    let mut stream = stream.unwrap();
    stream
        .send(tungstenite::Message::Text(
            serde_json::to_string(&request).unwrap(),
        ))
        .await
        .unwrap()
}

#[derive(Debug, Deserialize, Serialize)]
struct Login {
    code: RequestType,
    account: String,
    password: String,
    login_type: LoginType,
}

impl Login {
    fn new(account: String, password: String, login_type: LoginType) -> Self {
        Self {
            code: RequestType::Login,
            account,
            password,
            login_type,
        }
    }
}

async fn test_login() -> ClientWS {
    let user = get_test_user();
    let mut connection = connect_to_server_internal().await.unwrap();
    let login_req = Login::new(user.ocid.clone(), user.password.clone(), LoginType::Ocid);
    connection
        .send(tungstenite::Message::Text(
            serde_json::to_string(&login_req).unwrap(),
        ))
        .await
        .unwrap();
    connection
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

pub fn init_server() -> &'static ClientWS {
    static TMP: OnceLock<(TearDown, ClientWS)> = OnceLock::new();
    &TMP.get_or_init(|| {
        thread::spawn(|| set_up_server());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let conn = runtime.block_on(async {
            test_register().await;
            test_login().await
        });
        (TearDown {}, conn)
    })
    .1
}

pub type ClientWS = WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

async fn connect_to_server_internal() -> anyhow::Result<ClientWS> {
    let (ws, _) =
        tokio_tungstenite::connect_async(server::utils::ws_bind_addr().to_string()).await?;
    Ok(ws)
}

/// 连接到服务器
async fn get_connection() -> &'static ClientWS {
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
