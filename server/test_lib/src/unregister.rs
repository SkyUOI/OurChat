use futures_util::SinkExt;
use server::requests::Unregister;
use tungstenite::Message;

/// 清理测试环境时顺便测试帐号删除，删除需要在所有测试后运行，所以只能在这里测试
pub async fn test_unregister() {
    let conn = crate::get_connection();
    let req = Unregister::new();
    let mut lock = conn.lock().await;
    lock.send(Message::text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap()
}
