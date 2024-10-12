use crate::helper;
use server::{
    client::{
        requests,
        response::{InviteSession, NewSessionResponse},
    },
    consts::MessageType,
};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_session() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user1 = app.new_user_logined().await.unwrap();
    let user2 = app.new_user_logined().await.unwrap();
    let user3 = app.new_user().await.unwrap();
    // try to create a session in two users
    let req = requests::NewSession::new_easiest(vec![
        user2.lock().await.ocid.clone(),
        user3.lock().await.ocid.clone(),
    ]);
    user1
        .lock()
        .await
        .send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    // get new session response
    let resp = user1.lock().await.get().await.unwrap();
    let json: NewSessionResponse = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    assert_eq!(json.status, requests::Status::Success);
    assert_eq!(json.code, MessageType::NewSessionRes);
    let session_id = json.session_id.unwrap();
    // verify user2 received the invite
    let resp = user2.lock().await.get().await.unwrap();
    let json: InviteSession = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    assert_eq!(json.inviter_id, user1.lock().await.ocid);
    assert_eq!(json.code, MessageType::InviteSession);
    assert!(json.message.is_empty());
    assert_eq!(json.session_id, session_id);

    // verify user3 received the invite
    user3.lock().await.ocid_login().await.unwrap();
    let resp = user3.lock().await.get().await.unwrap();
    dbg!(&resp);
    let json: InviteSession = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    assert_eq!(json.inviter_id, user1.lock().await.ocid);
    assert_eq!(json.code, MessageType::InviteSession);
    assert!(json.message.is_empty());
    assert_eq!(json.session_id, session_id);
    app.async_drop().await;
}
