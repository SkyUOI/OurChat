use crate::helper;
use server::{
    client::{requests, response::NewSessionResponse},
    consts::MessageType,
};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_session() {
    let mut app = helper::TestApp::new_logined(None).await.unwrap();
    let (user1, mut conn1) = app.new_user().await.unwrap();
    let (user2, mut conn2) = app.new_user().await.unwrap();
    // try to create a session in two users
    let req = requests::NewSession::new_easiest(vec![user1.ocid]);
    app.send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    // get new session response
    let resp = app.get().await.unwrap();
    dbg!(resp.to_text().unwrap());
    let json: NewSessionResponse = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    assert_eq!(json.status, requests::Status::Success);
    assert_eq!(json.code, MessageType::NewSessionRes);
    conn1.close(None).await.unwrap();
    conn2.close(None).await.unwrap();
    app.async_drop().await;
}
