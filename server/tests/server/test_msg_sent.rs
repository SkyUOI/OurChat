use crate::helper;
use server::{
    client::{
        MsgConvert,
        basic::{TextMsg, UnitMsg},
        requests::{GetUserMsgRequest, UserSendMsgRequest},
        response::{GetUserMsgResponse, UserSendMsgResponse},
    },
    consts::MessageType,
};

#[tokio::test]
async fn test_text_sent() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let (session_user, session) = app.new_session(3, "session1").await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let time = app.get_timestamp().await;
    let msg_sent = UserSendMsgRequest::new(session.session_id, time, vec![UnitMsg::Text(
        TextMsg::new("hello".into()),
    )]);
    a.lock().await.send(msg_sent.to_msg()).await.unwrap();
    let resp =
        UserSendMsgResponse::from_json(a.lock().await.recv().await.unwrap().to_text().unwrap())
            .unwrap();
    let msg_id = resp.msg_id;
    app.async_drop().await;
}

#[tokio::test]
async fn test_text_get() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let (session_user, session) = app.new_session(3, "session1").await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let base_time = app.get_timestamp().await;
    // send message
    let msg_should_sent = UnitMsg::Text(TextMsg::new("hello".into()));
    let time = app.get_timestamp().await;
    let msg_sent = UserSendMsgRequest::new(session.session_id, time, vec![msg_should_sent.clone()]);
    a.lock().await.send(msg_sent.to_msg()).await.unwrap();
    let resp =
        UserSendMsgResponse::from_json(a.lock().await.recv().await.unwrap().to_text().unwrap())
            .unwrap();
    let msg_id = resp.msg_id;
    let msg_sent = UserSendMsgRequest::new(session.session_id, time, vec![msg_should_sent.clone()]);
    b.lock().await.send(msg_sent.to_msg()).await.unwrap();
    let resp =
        UserSendMsgResponse::from_json(b.lock().await.recv().await.unwrap().to_text().unwrap())
            .unwrap();
    // get message
    let msg_get = GetUserMsgRequest::new(base_time);
    c.lock().await.send(msg_get.to_msg()).await.unwrap();
    let resp =
        GetUserMsgResponse::from_json(c.lock().await.recv().await.unwrap().to_text().unwrap())
            .unwrap();
    assert_eq!(resp.code, MessageType::GetBundledUserMsgRes);
    assert_eq!(resp.session_id, session.session_id);
    assert_eq!(resp.msgs.len(), 2);
    assert_eq!(resp.msgs[0].bundle_msg, vec![msg_should_sent.clone()]);
    assert_eq!(resp.msgs[1].bundle_msg, vec![msg_should_sent]);
    app.async_drop().await;
}
