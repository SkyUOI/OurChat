use client::TestApp;
use pb::ourchat::msg_delivery::{self, recall::v1::RecallMsgRequest, v1::OneMsg};

#[tokio::test]
async fn test_recall() {
    // TODO:test whether recall signal is received
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    // Send Msg
    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, vec![OneMsg {
            data: Some(msg_delivery::v1::one_msg::Data::Text("hello".to_owned())),
        }])
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;
    // Recall Back
    let ret = a
        .lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest { msg_id })
        .await
        .unwrap();
    app.async_drop().await;
}
