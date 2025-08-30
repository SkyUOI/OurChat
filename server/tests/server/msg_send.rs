use std::collections::HashSet;

use pb::service::ourchat::msg_delivery::{
    self,
    v1::{OneMsg, fetch_msgs_response, one_msg},
};
use sea_orm::prelude::DateTimeUtc;

#[tokio::test]
async fn test_text_sent() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, _b, _cc) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let ret: tonic::Response<msg_delivery::v1::SendMsgResponse> = a
        .lock()
        .await
        .send_msg(
            session.session_id,
            vec![OneMsg {
                data: Some(one_msg::Data::Text("hello".to_owned())),
            }],
            false,
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
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, _b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    // send a message
    let msg_should_sent = OneMsg {
        data: Some(one_msg::Data::Text("hello".to_owned())),
    };
    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, vec![msg_should_sent.clone()], false)
        .await
        .unwrap();
    let mut msg_id = vec![ret.into_inner().msg_id];

    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, vec![msg_should_sent.clone()], false)
        .await
        .unwrap()
        .into_inner();
    msg_id.push(ret.msg_id);

    let msgs = c.lock().await.fetch_msgs().fetch(2).await.unwrap();
    for (i, msg_id) in msgs.into_iter().zip(msg_id.iter()) {
        if let fetch_msgs_response::RespondEventType::Msg(ref item) = i.respond_event_type.unwrap()
        {
            assert_eq!(item.session_id, u64::from(session.session_id));
            assert_eq!(item.bundle_msgs, vec![msg_should_sent.clone()]);
            assert_eq!(i.msg_id, *msg_id);
        }
    }
    c.lock()
        .await
        .fetch_msgs()
        .set_timestamp(DateTimeUtc::from_timestamp_nanos(0))
        .fetch(2)
        .await
        .unwrap();
    let msg_sent_myself = a
        .lock()
        .await
        .fetch_msgs()
        .fetch(1)
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    if let fetch_msgs_response::RespondEventType::Msg(ref item) =
        msg_sent_myself.respond_event_type.unwrap()
    {
        assert_eq!(item.session_id, u64::from(session.session_id));
        assert_eq!(item.bundle_msgs, vec![msg_should_sent.clone()]);
        assert_eq!(msg_sent_myself.msg_id, msg_id[0]);
    }
    app.async_drop().await;
}

#[tokio::test]
async fn test_repeated_msg() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    a.lock()
        .await
        .send_msg(
            session.session_id,
            vec![OneMsg {
                data: Some(one_msg::Data::Text("hello".to_owned())),
            }],
            false,
        )
        .await
        .unwrap();
    b.lock()
        .await
        .send_msg(
            session.session_id,
            vec![OneMsg {
                data: Some(one_msg::Data::Text("hello".to_owned())),
            }],
            false,
        )
        .await
        .unwrap();
    let check = async |msgs: Vec<msg_delivery::v1::FetchMsgsResponse>| {
        let mut users = HashSet::from([b.lock().await.id, a.lock().await.id]);
        for i in msgs {
            if let fetch_msgs_response::RespondEventType::Msg(ref item) =
                i.respond_event_type.unwrap()
            {
                assert!(users.contains(&item.sender_id.into()));
                users.remove(&item.sender_id.into());
            }
        }
    };
    for _ in 0..10 {
        let msgs = c
            .lock()
            .await
            .fetch_msgs()
            .set_timestamp(DateTimeUtc::from_timestamp_nanos(0))
            .fetch(2)
            .await
            .unwrap();
        check(msgs).await;
    }
    for _ in 0..10 {
        let msgs = a
            .lock()
            .await
            .fetch_msgs()
            .set_timestamp(DateTimeUtc::from_timestamp_nanos(0))
            .fetch(2)
            .await
            .unwrap();
        check(msgs).await;
    }
    for _ in 0..10 {
        let msgs = c
            .lock()
            .await
            .fetch_msgs()
            .set_timestamp(DateTimeUtc::from_timestamp_nanos(0))
            .fetch(2)
            .await
            .unwrap();
        check(msgs).await;
    }
    for _ in 0..10 {
        let msgs = c
            .lock()
            .await
            .fetch_msgs()
            .set_timestamp(DateTimeUtc::from_timestamp_nanos(0))
            .fetch(2)
            .await
            .unwrap();
        check(msgs).await;
        let msgs = a
            .lock()
            .await
            .fetch_msgs()
            .set_timestamp(DateTimeUtc::from_timestamp_nanos(0))
            .fetch(2)
            .await
            .unwrap();
        check(msgs).await;
    }
    app.async_drop().await;
}
