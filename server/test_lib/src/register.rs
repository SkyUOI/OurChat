use crate::{connect_to_server_internal, get_test_user};
use futures_util::{SinkExt, StreamExt};
use server::requests::Register;
use std::thread;

/// 在这里测试注册顺便初始化服务器，注册需要在所有测试前运行，所以只能在这里测试
pub(crate) async fn test_register() {
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
        .unwrap();
    let ret = stream.next().await.unwrap().unwrap();
    println!("{}", ret);
    stream.close(None).await.unwrap();
}
