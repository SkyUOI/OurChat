use super::{create_connection, TEST_USER};
use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response,
    consts::MessageType,
    requests::{self, Register},
};
use std::thread;
use tokio_tungstenite::tungstenite::Message;

/// 在这里测试注册顺便初始化服务器，注册需要在所有测试前运行，所以只能在这里测试
pub(crate) async fn test_register() -> String {
    let request = Register::new(
        TEST_USER.name.clone(),
        TEST_USER.password.clone(),
        TEST_USER.email.clone(),
    );
    let mut stream = None;
    // 服务器启动可能没那么快
    for i in 0..10 {
        eprintln!("Try to connect to server:{}", i);
        let ret = create_connection().await;
        if ret.is_ok() {
            stream = ret.ok();
            break;
        }
        if i == 9 {
            panic!("Cannot connect to server");
        }
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    let mut stream = stream.unwrap();
    stream
        .send(Message::Text(serde_json::to_string(&request).unwrap()))
        .await
        .unwrap();
    let ret = stream.next().await.unwrap().unwrap();
    stream.close(None).await.unwrap();
    let json: client_response::RegisterResponse = serde_json::from_str(&ret.to_string()).unwrap();
    assert_eq!(json.status, requests::Status::Success);
    assert_eq!(json.code, MessageType::RegisterRes);
    json.ocid.unwrap()
}
