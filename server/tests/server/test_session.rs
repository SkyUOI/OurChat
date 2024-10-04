use crate::helper;
use server::client::requests;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_session() {
    let mut app = helper::TestApp::new_logined(None).await.unwrap();
    let (user1, conn1) = app.new_user().await.unwrap();
    let (user2, conn2) = app.new_user().await.unwrap();
    // try to create a session in two users
    let req = requests::NewSession::new_easiest(vec![app.user.ocid.clone(), user1.ocid]);
    app.send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    app.async_drop().await;
}
