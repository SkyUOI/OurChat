use crate::helper;
use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response::get_status::GetStatusResponse, requests::get_status::GetStatus,
};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_status() {
    let mut app = helper::TestApp::new_logined().await.unwrap();
    let req = GetStatus::new();
    app.connection
        .send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    let ret = app.connection.next().await.unwrap().unwrap();
    assert_eq!(
        ret,
        Message::Text(serde_json::to_string(&GetStatusResponse::normal()).unwrap())
    );
    app.async_drop().await;
}
