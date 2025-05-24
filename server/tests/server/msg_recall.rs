use claims::assert_lt;
use client::TestApp;
use parking_lot::Mutex;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use pb::service::ourchat::msg_delivery::{self, recall::v1::RecallMsgRequest, v1::OneMsg};
use pb::time::from_google_timestamp;
use std::sync::Arc;
use std::time::Duration;
use tokio::join;
use tokio::sync::{Notify, oneshot};

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
        .send_msg(
            session.session_id,
            vec![OneMsg {
                data: Some(msg_delivery::v1::one_msg::Data::Text("hello".to_owned())),
            }],
        )
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;
    // start a listening process
    let res = Arc::new(Mutex::new(None));
    let res_clone = res.clone();
    let c_clone = c.clone();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let (tx, rx) = oneshot::channel();

    let task = tokio::spawn(async move {
        tx.send(()).unwrap();
        let ret = c_clone
            .lock()
            .await
            .fetch_msgs_notify(notify_clone)
            .await
            .unwrap();
        *res_clone.lock() = Some(ret);
    });
    rx.await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
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
    let b_rec = b.lock().await.fetch_msgs(1).await.unwrap();
    let check = |rec: Vec<FetchMsgsResponse>, msg_len, msg_recall_idx: usize| {
        assert_eq!(rec.len(), msg_len, "{rec:?}");
        assert_lt!(
            from_google_timestamp(&rec[msg_recall_idx].time.unwrap()).unwrap(),
            chrono::Utc::now()
        );
        assert_eq!(rec[msg_recall_idx].msg_id, recall_msg_id);
        let RespondMsgType::Recall(data) = rec[msg_recall_idx].clone().respond_msg_type.unwrap()
        else {
            panic!("not a recall notification")
        };
        assert_eq!(data.msg_id, msg_id);
    };
    check(b_rec, 1, 0);
    notify.notify_waiters();
    join!(task).0.unwrap();
    check(res.lock().clone().unwrap(), 2, 1);
    app.async_drop().await;
}
