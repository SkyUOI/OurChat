use crate::UsersGroup;
use crate::framework::{Record, Report, StressTest, run_user_stress_test};
use base::consts::SessionID;
use dashmap::DashMap;
use pb::service::ourchat::msg_delivery::recall::v1::RecallMsgRequest;
use pb::service::ourchat::msg_delivery::v1::{FetchMsgsRequest, SendMsgRequest};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use derive::register_test;

#[register_test("Send Message", WithSessions)]
pub async fn test_send_msg(
    sessions: Arc<DashMap<base::consts::ID, SessionID>>,
    users: &UsersGroup,
    report: &mut Report,
) -> anyhow::Result<Arc<DashMap<base::consts::ID, u64>>> {
    tracing::info!("▶️  Running test: 'send_msg'");
    let mut stress_test = StressTest::builder().set_concurrency(100).set_requests(100);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let msg_ids = Arc::new(DashMap::new());
    let msg_ids_ret = msg_ids.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let sessions = sessions.clone();
            let msg_ids = msg_ids.clone();
            async move {
                // Get user_id and check session in one lock to avoid deadlock
                let (user_id, session_id) = {
                    let u = user.lock().await;
                    let user_id = u.id;
                    let session_id = sessions.get(&user_id).map(|s| *s);
                    (user_id, session_id)
                };
                if let Some(session_id) = session_id {
                    match user
                        .lock()
                        .await
                        .oc()
                        .send_msg(SendMsgRequest {
                            session_id: session_id.0,
                            markdown_text: format!("Test message {}", rand::random::<u32>()),
                            involved_files: vec![],
                            is_encrypted: false,
                        })
                        .await
                    {
                        Ok(resp) => {
                            let msg_id = resp.into_inner().msg_id;
                            msg_ids.insert(user_id, msg_id);
                            true
                        }
                        Err(_) => false,
                    }
                } else {
                    false
                }
            }
        })
        .await;
    report.add_record(Record::new("send_msg", output));
    Ok(msg_ids_ret)
}

#[register_test("Fetch Messages", WithUsers)]
pub async fn test_fetch_msgs(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "fetch_msgs",
        users,
        100,
        100,
        |user, _now, _users| async move {
            user.lock()
                .await
                .oc()
                .fetch_msgs(FetchMsgsRequest { time: None })
                .await
                .is_ok()
        },
    )
    .await;
}

#[register_test("Recall Message", WithSessions)]
pub async fn test_recall(
    sessions: Arc<DashMap<base::consts::ID, SessionID>>,
    msg_ids: Arc<DashMap<base::consts::ID, u64>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    tracing::info!("▶️  Running test: 'recall'");
    let mut stress_test = StressTest::builder().set_concurrency(100).set_requests(100);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let sessions = sessions.clone();
            let msg_ids = msg_ids.clone();
            async move {
                // Get user_id and check session/msg in one lock to avoid deadlock
                let (session_id, msg_id) = {
                    let u = user.lock().await;
                    let user_id = u.id;
                    let session_id = sessions.get(&user_id).map(|s| *s);
                    let msg_id = msg_ids.get(&user_id).map(|m| *m);
                    (session_id, msg_id)
                };
                if let (Some(session_id), Some(msg_id)) = (session_id, msg_id) {
                    user.lock()
                        .await
                        .oc()
                        .recall_msg(RecallMsgRequest {
                            msg_id,
                            session_id: session_id.0,
                        })
                        .await
                        .is_ok()
                } else {
                    false
                }
            }
        })
        .await;
    report.add_record(Record::new("recall", output));
}
