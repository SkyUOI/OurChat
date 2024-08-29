use super::{create_connection, ClientWS, TEST_USER};
use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response::{self, ErrorMsgResponse},
    consts::MessageType,
    requests::{Login, LoginType},
};
use tokio_tungstenite::tungstenite::protocol::Message;

// 测试ocid登录
async fn test_ocid_login(ocid: String, mut connection: ClientWS) {
    let login_req = Login::new(ocid, TEST_USER.password.clone(), LoginType::Ocid);
    connection
        .send(Message::Text(serde_json::to_string(&login_req).unwrap()))
        .await
        .unwrap();
    let ret = connection.next().await.unwrap().unwrap();
    let json: client_response::LoginResponse =
        serde_json::from_str(ret.to_text().unwrap()).unwrap();
    assert_eq!(json.code, MessageType::LoginRes);
    connection.close(None).await.unwrap()
}

async fn test_email_login(ocid: String) -> ClientWS {
    let mut connection = create_connection().await.unwrap();
    let login_req = Login::new(
        TEST_USER.email.clone(),
        TEST_USER.password.clone(),
        LoginType::Email,
    );
    connection
        .send(Message::Text(serde_json::to_string(&login_req).unwrap()))
        .await
        .unwrap();
    let ret = connection.next().await.unwrap().unwrap();
    let json: client_response::LoginResponse =
        serde_json::from_str(ret.to_text().unwrap()).unwrap();
    assert_eq!(json.code, MessageType::LoginRes);
    assert_eq!(json.ocid.unwrap(), ocid);
    connection
}

/// 登录失败
async fn failed_login(conn: &mut ClientWS) {
    let wrong_msg = r#"{"code":65536}"#;
    conn.send(Message::Text(wrong_msg.to_string()))
        .await
        .unwrap();
    let ret = conn.next().await.unwrap().unwrap();
    let _: ErrorMsgResponse = serde_json::from_str(ret.to_text().unwrap()).unwrap();
}

pub(crate) async fn test_login(ocid: String) -> ClientWS {
    let mut connection = create_connection().await.unwrap();
    failed_login(&mut connection).await;
    test_ocid_login(ocid.clone(), connection).await;
    test_email_login(ocid).await
}
