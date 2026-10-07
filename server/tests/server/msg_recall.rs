use claims::assert_lt;
use client::TestApp;
use client::oc_helper::user::wait_for_response_in_sink;
use parking_lot::Mutex;
use pb::service::ourchat::msg_delivery::recall::v1::RecallMsgRequest;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::time::TimeStampUtc;
use std::sync::Arc;
use std::time::Duration;
use tokio::join;
use tokio::sync::{Notify, oneshot};

/// Move the timestamp of a message `ago` into the past, to simulate an old message
async fn backdate_msg(app: &TestApp, msg_id: u64, ago: Duration) {
    use sea_orm::{ConnectionTrait, Statement};
    let old_time = chrono::Utc::now() - ago;
    let values: [sea_orm::Value; 2] = [old_time.into(), (msg_id as i64).into()];
    let ret = app
        .get_db_connection()
        .execute_raw(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "UPDATE message_records SET time = $1 WHERE msg_id = $2",
            values,
        ))
        .await
        .unwrap();
    assert_eq!(ret.rows_affected(), 1, "backdate_msg matched no message");
}

#[tokio::test]
async fn test_recall() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    // Listener on c that collects every arriving event into a shared sink,
    // so the test can watch the collection while the stream stays open.
    let c_msgs: Arc<Mutex<Vec<FetchMsgsResponse>>> = Arc::new(Mutex::new(vec![]));
    let sink = c_msgs.clone();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let c_clone = c.clone();
    let (tx, rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        tx.send(()).unwrap();
        let _ = c_clone
            .lock()
            .await
            .fetch_msgs()
            .fetch_stream_with_sink(sink, notify_clone)
            .await;
    });
    rx.await.unwrap();
    // Readiness: a probe message recalled instantly can only appear in the
    // sink through LIVE delivery (its recall deletes it from history), so
    // observing one proves c's consumer is bound — no fixed sleeps, works
    // on any runner speed.
    let mut live = false;
    for _ in 0..20 {
        if a.lock()
            .await
            .probe_live_delivery(session.session_id, &c_msgs)
            .await
            .unwrap()
        {
            live = true;
            break;
        }
    }
    assert!(live, "listener never received a live probe");

    // Everything after this point is deterministic.
    let marker: TimeStampUtc = chrono::Utc::now();
    // Send Msg
    let ret = a
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;
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
    // receive the recall signal: b's fresh fetch is pinned to a window after
    // the probes, and the recall deleted "hello" from history, so exactly
    // [recall] replays.
    let b_rec = b
        .lock()
        .await
        .fetch_msgs()
        .set_timestamp(marker)
        .fetch(1)
        .await
        .unwrap();
    let check = async |rec: Vec<FetchMsgsResponse>, msg_len, msg_recall_idx: usize| {
        assert_eq!(rec.len(), msg_len, "{rec:?}");
        tokio::time::sleep(Duration::from_millis(200)).await;
        let tmp: TimeStampUtc = rec[msg_recall_idx].time.unwrap().try_into().unwrap();
        assert_lt!(tmp, chrono::Utc::now());
        assert_eq!(rec[msg_recall_idx].msg_id, recall_msg_id);
        let RespondEventType::Recall(data) =
            rec[msg_recall_idx].clone().respond_event_type.unwrap()
        else {
            panic!("not a recall notification")
        };
        assert_eq!(data.msg_id, msg_id);
    };
    check(b_rec, 1, 0).await;
    // c received both through the live path, message before recall. The
    // recall's arrival on b's fresh stream only proves it was PUBLISHED; c's
    // live delivery (broker -> consumer -> grpc -> sink) is not ordered with
    // that, so wait for it to land in c's sink BEFORE closing the listener —
    // the same queue is FIFO, so hello is already there too.
    wait_for_response_in_sink(
        &c_msgs,
        |m| (m.msg_id == recall_msg_id).then_some(()),
        Duration::from_secs(20),
    )
    .await
    .expect("listener never received the recall through the live stream");
    notify.notify_waiters();
    join!(task).0.unwrap();
    let tmp = c_msgs.lock().clone();
    let hello_idx = tmp
        .iter()
        .position(|m| m.msg_id == msg_id)
        .expect("listener missed the message");
    let recall_idx = tmp
        .iter()
        .position(|m| m.msg_id == recall_msg_id)
        .expect("listener missed the recall");
    assert!(recall_idx > hello_idx, "{tmp:?}");
    check(vec![tmp[recall_idx].clone()], 1, 0).await;
    app.async_drop().await;
}

#[tokio::test]
async fn test_recall_time_limit_exceeded() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    // `session_user[0]` is the session Owner, so the message must be sent by the non-privileged user 1
    let (b, c) = (session_user[1].clone(), session_user[2].clone());
    let ret = b
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;
    // Simulate an old message (the default recall_time_limit is 2m)
    backdate_msg(&app, msg_id, Duration::from_secs(10 * 60)).await;
    // The sender can no longer recall their own message
    let e = b
        .lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::OutOfRange, "{e:?}");
    // A user without the RecallMsg permission cannot recall another one's old message
    let e = c
        .lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied, "{e:?}");
    app.async_drop().await;
}

#[tokio::test]
async fn test_recall_time_limit_permission_exempt() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, _c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    // b sends a message
    let ret = b
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;
    // Simulate an old message (the default recall_time_limit is 2m)
    backdate_msg(&app, msg_id, Duration::from_secs(10 * 60)).await;
    // a is the Owner of the session, whose RecallMsg permission is not bound by the time limit
    a.lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();
    app.async_drop().await;
}

#[tokio::test]
async fn test_recall_no_time_limit() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    // Disable the time limit
    config.main_cfg.recall_time_limit = Duration::ZERO;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    // `session_user[0]` is the session Owner, so the message must be sent by the non-privileged user 1
    let b = session_user[1].clone();
    // Send Msg
    let ret = b
        .lock()
        .await
        .send_msg(session.session_id, "hello", vec![], false)
        .await
        .unwrap();
    let msg_id = ret.into_inner().msg_id;
    // Simulate an old message
    backdate_msg(&app, msg_id, Duration::from_secs(10 * 60)).await;
    // The sender can still recall their own message when the limit is disabled
    b.lock()
        .await
        .oc()
        .recall_msg(RecallMsgRequest {
            msg_id,
            session_id: session.session_id.into(),
        })
        .await
        .unwrap();
    app.async_drop().await;
}
