use base::time::from_google_timestamp;
use claims::assert_gt;
use pb::service::ourchat::msg_delivery::{
    self,
    v1::{OneMsg, fetch_msgs_response},
};
use std::time::Duration;

#[tokio::test]
async fn test_text_sent() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let (a, _b, _cc) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let ret = a
        .lock()
        .await
        .send_msg(
            session.session_id,
            vec![OneMsg {
                data: Some(msg_delivery::v1::one_msg::Data::Text("hello".to_owned())),
            }],
        )
        .await
        .unwrap();
    let _msg_id = ret.into_inner().msg_id;
    app.async_drop().await;
}

#[tokio::test]
async fn test_text_get() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let (a, _b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let base_time = app.get_timestamp().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    // send a message
    let msg_should_sent = OneMsg {
        data: Some(msg_delivery::v1::one_msg::Data::Text("hello".to_owned())),
    };
    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, vec![msg_should_sent.clone()])
        .await
        .unwrap();
    let mut msg_id = vec![ret.into_inner().msg_id];

    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, vec![msg_should_sent.clone()])
        .await
        .unwrap()
        .into_inner();
    msg_id.push(ret.msg_id);

    let msgs = c
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(400))
        .await
        .unwrap();
    assert_eq!(msgs.len(), 2);
    for (i, msg_id) in msgs.into_iter().zip(msg_id.iter()) {
        assert_gt!(from_google_timestamp(&i.time.unwrap()).unwrap(), base_time);
        if let fetch_msgs_response::RespondMsgType::Msg(ref item) = i.respond_msg_type.unwrap() {
            assert_eq!(item.session_id, u64::from(session.session_id));
            assert_eq!(item.bundle_msgs, vec![msg_should_sent.clone()]);
            assert_eq!(i.msg_id, *msg_id);
        }
    }
    app.async_drop().await;
}
