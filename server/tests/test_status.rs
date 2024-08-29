mod test_lib;

use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response::get_status::GetStatusResponse, requests::get_status::GetStatus,
};
use tokio_tungstenite::tungstenite::Message;

async fn test_status() {
    let conn = test_lib::get_connection().await;
    let req = GetStatus::new();
    let mut lock = conn.lock().await;
    lock.send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    let ret = lock.next().await.unwrap().unwrap();
    assert_eq!(
        ret,
        Message::Text(serde_json::to_string(&GetStatusResponse::normal()).unwrap())
    );
}

register_test!(test_status);
