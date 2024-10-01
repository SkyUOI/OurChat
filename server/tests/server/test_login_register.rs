use crate::helper::{self, ClientWS};
use futures_util::{SinkExt, StreamExt};
use server::connection::client_response::ErrorMsgResponse;
use tokio_tungstenite::tungstenite::Message;

/// 登录失败
async fn failed_login(conn: &mut ClientWS) {
    let wrong_msg = r#"{"code":65536}"#;
    conn.send(Message::Text(wrong_msg.to_string()))
        .await
        .unwrap();
    let ret = conn.next().await.unwrap().unwrap();
    let _: ErrorMsgResponse = serde_json::from_str(ret.to_text().unwrap()).unwrap();
}

#[tokio::test]
async fn test_email_login() {
    let mut app = helper::TestApp::new().await.unwrap();
    failed_login(&mut app.connection).await;
    app.email_login().await;
    app.async_drop().await;
}

#[tokio::test]
async fn test_ocid_login() {
    let mut app = helper::TestApp::new().await.unwrap();
    failed_login(&mut app.connection).await;
    app.ocid_login().await;
    app.async_drop().await;
}
