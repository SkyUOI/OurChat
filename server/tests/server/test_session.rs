use pb::ourchat::session::new_session::v1::NewSessionRequest;

#[tokio::test]
async fn test_session() {
    let mut app = client::TestApp::new_with_launching_instance(None)
        .await
        .unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user3 = app.new_user().await.unwrap();
    // try to create a session in two users
    let req = NewSessionRequest {
        members: vec![
            user2.lock().await.ocid.clone(),
            user3.lock().await.ocid.clone(),
        ],
        ..Default::default()
    };
    // get new session response
    let ret = user1.lock().await.oc().new_session(req).await.unwrap();
    let ret = ret.into_inner();
    let session_id = ret.session_id;
    // verify user2 received the invite
    // let resp = user2.lock().await.recv().await.unwrap();
    // let json: InviteSession = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    // assert_eq!(json.inviter_id, user1.lock().await.ocid);
    // assert_eq!(json.code, MessageType::InviteSession);
    // assert!(json.message.is_empty());
    // assert_eq!(json.session_id, session_id);

    // verify user3 received the invite
    // user3.lock().await.ocid_login().await.unwrap();
    // let resp = user3.lock().await.recv().await.unwrap();
    // dbg!(&resp);
    // let json: InviteSession = serde_json::from_str(resp.to_text().unwrap()).unwrap();
    // assert_eq!(json.inviter_id, user1.lock().await.ocid);
    // assert_eq!(json.code, MessageType::InviteSession);
    // assert!(json.message.is_empty());
    // assert_eq!(json.session_id, session_id);
    app.async_drop().await;
}
