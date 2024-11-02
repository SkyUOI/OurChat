use crate::helper;
use server::client::{
    MsgConvert,
    basic::{Msg, TextMsg},
    requests::UserSendMsgRequest,
    response::UserSendMsgResponse,
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
    let msg_sent = UserSendMsgRequest::new(session.session_id, time, vec![Msg::Text(
        TextMsg::new("hello".into()),
    )]);
    a.lock().await.send(msg_sent.to_msg()).await.unwrap();
    let resp =
        UserSendMsgResponse::from_json(a.lock().await.recv().await.unwrap().to_text().unwrap())
            .unwrap();
    let msg_id = resp.msg_id;
    app.async_drop().await;
}
