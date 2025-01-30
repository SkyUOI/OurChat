use base::time::from_google_timestamp;
use claims::assert_lt;
use client::TestApp;
use parking_lot::Mutex;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use pb::service::ourchat::msg_delivery::{self, recall::v1::RecallMsgRequest, v1::OneMsg};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_recall() {
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
    // start a listening process
    let res = Arc::new(Mutex::new(None));
    let res_clone = res.clone();
    let c_clone = c.clone();
    tokio::spawn(async move {
        let ret = c_clone
            .lock()
            .await
            .fetch_msgs(Duration::from_millis(200))
            .await
            .unwrap();
        *res_clone.lock() = Some(ret);
    });
    // Recall Back
    let recall_msg = a
        .lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner();
    let recall_msg_id = recall_msg.msg_id;
    // receive the recall signal
    let b_rec = b
        .lock()
        .await
        .fetch_msgs(Duration::from_millis(200))
        .await
        .unwrap();
    let check = |rec: Vec<FetchMsgsResponse>| {
        assert_eq!(rec.len(), 1);
        assert_lt!(
            from_google_timestamp(&rec[0].time.unwrap()).unwrap(),
            chrono::Utc::now()
        );
        assert_eq!(rec[0].msg_id, recall_msg_id);
        let RespondMsgType::Recall(data) = rec[0].clone().respond_msg_type.unwrap() else {
            panic!("not a recall notification")
        };
        assert_eq!(data.msg_id, msg_id);
    };
    check(b_rec);
    check(res.lock().clone().unwrap());
    app.async_drop().await;
}
