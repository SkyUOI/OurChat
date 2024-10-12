use crate::helper::{self, ClientWS};
use claims::assert_ok;
use futures_util::{SinkExt, StreamExt};
use server::client::response::ErrorMsgResponse;
use tokio_tungstenite::tungstenite::Message;

/// Login failed
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
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    failed_login(user.lock().await.get_conn()).await;
    assert_ok!(user.lock().await.email_login().await);
    app.async_drop().await;
}

#[tokio::test]
async fn test_ocid_login() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    failed_login(user.lock().await.get_conn()).await;
    assert_ok!(user.lock().await.ocid_login().await);
    app.async_drop().await;
}
