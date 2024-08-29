use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response::UnregisterResponse,
    consts::MessageType,
    requests::{self, Unregister},
};
use tokio_tungstenite::tungstenite::Message;

/// 清理测试环境时顺便测试帐号删除，删除需要在所有测试后运行，所以只能在这里测试
pub async fn test_unregister() {
    let conn = super::get_connection().await;
    let req = Unregister::new();
    let mut lock = conn.lock().await;
    lock.send(Message::text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    let ret = (*lock).next().await.unwrap().unwrap();
    let json: UnregisterResponse = serde_json::from_str(ret.to_text().unwrap()).unwrap();
    assert_eq!(json.code, MessageType::UnregisterRes);
    assert_eq!(json.status, requests::Status::Success);
}
