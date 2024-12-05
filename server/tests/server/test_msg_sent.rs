use base::time::to_google_timestamp;
use pb::ourchat::msg_delivery::{
    self,
    v1::{FetchMsgsRequest, OneMsg, SendMsgRequest, fetch_msgs_response},
};
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_text_sent() {
    let mut app = client::TestApp::new_with_launching_instance(None)
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let time = app.get_timestamp().await;
    let time_google = to_google_timestamp(time);
    let msg_sent = SendMsgRequest {
        session_id: session.session_id.into(),
        time: Some(time_google),
        bundle_msgs: vec![OneMsg {
            data: Some(msg_delivery::v1::one_msg::Data::Text("hello".to_owned())),
        }],
    };
    let ret = a.lock().await.oc().send_msg(msg_sent).await.unwrap();
    let msg_id = ret.into_inner().msg_id;
    app.async_drop().await;
}

#[tokio::test]
async fn test_text_get() {
    let mut app = client::TestApp::new_with_launching_instance(None)
        .await
        .unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let base_time = app.get_timestamp().await;
    // send message
    let msg_should_sent = OneMsg {
        data: Some(msg_delivery::v1::one_msg::Data::Text("hello".to_owned())),
    };
    let time = app.get_timestamp().await;
    let time_google = to_google_timestamp(time);
    let msg_sent = SendMsgRequest {
        session_id: session.session_id.into(),
        time: Some(time_google),
        bundle_msgs: vec![msg_should_sent.clone()],
    };
    let ret = a.lock().await.oc().send_msg(msg_sent).await.unwrap();
    let mut msg_id = vec![ret.into_inner().msg_id];

    let time = app.get_timestamp().await;
    let time_google = to_google_timestamp(time);
    let msg_sent = SendMsgRequest {
        session_id: session.session_id.into(),
        time: Some(time_google),
        bundle_msgs: vec![msg_should_sent.clone()],
    };
    let ret = a.lock().await.oc().send_msg(msg_sent).await.unwrap();
    msg_id.push(ret.into_inner().msg_id);

    // get message
    let msg_get = FetchMsgsRequest {
        time: Some(to_google_timestamp(base_time)),
    };
    let ret = c.lock().await.oc().fetch_msgs(msg_get).await.unwrap();
    let mut ret_stream = ret.into_inner();
    let mut msgs = vec![];
    while let Some(i) = tokio::select! {
        i = ret_stream.next() => i,
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => None
    } {
        msgs.push(i.unwrap().data.unwrap());
    }
    drop(ret_stream);
    assert_eq!(msgs.len(), 2);
    for (i, msg_id) in msgs.into_iter().zip(msg_id.iter()) {
        if let fetch_msgs_response::Data::Msg(i) = i {
            assert_eq!(i.session_id, u64::from(session.session_id));
            assert_eq!(i.bundle_msgs, vec![msg_should_sent.clone()]);
            assert_eq!(i.msg_id, *msg_id);
        }
    }
    app.async_drop().await;
}
