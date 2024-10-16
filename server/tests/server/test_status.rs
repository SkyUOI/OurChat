use crate::helper;
use server::client::{requests::get_status::GetStatus, response::get_status::GetStatusResponse};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_status() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user = app.new_user_logined().await.unwrap();
    let req = GetStatus::new();
    user.lock()
        .await
        .send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    let ret = user.lock().await.get().await.unwrap();
    assert_eq!(
        ret,
        Message::Text(serde_json::to_string(&GetStatusResponse::normal()).unwrap())
    );
    app.async_drop().await;
}
