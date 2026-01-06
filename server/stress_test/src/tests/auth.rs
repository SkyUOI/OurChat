use crate::UsersGroup;
use crate::framework::{Record, Report, StressTest, run_user_stress_test};
use pb::service::ourchat::get_account_info::v1::GetAccountInfoRequest;
use pb::service::ourchat::set_account_info::v1::SetSelfInfoRequest;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use derive::register_test;

#[register_test("User Registration", AppOnly)]
pub async fn test_register(report: &mut Report, app: &mut client::ClientCore) -> UsersGroup {
    tracing::info!("▶️  Running test: 'register'");
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);

    // Create users in parallel for better performance
    let mut set = tokio::task::JoinSet::new();
    for _ in 0..1000 {
        let app = app.clone();
        set.spawn(async move {
            Arc::new(tokio::sync::Mutex::new(
                client::oc_helper::user::TestUser::random_unreadable(&app).await,
            ))
        });
    }
    let mut users = Vec::with_capacity(1000);
    while let Some(result) = set.join_next().await {
        if let Ok(user) = result {
            users.push(user);
        }
    }
    let idx = Arc::new(AtomicUsize::new(0));
    let users = users;
    let users_test = users.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            async move {
                match user.lock().await.register().await {
                    Ok(_) => true,
                    Err(e) => {
                        tracing::error!("{}", e);
                        false
                    }
                }
            }
        })
        .await;

    report.add_record(Record::new("register", output));
    users_test
}

#[register_test("Authentication", WithUsers)]
pub async fn test_auth(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "auth",
        users,
        1000,
        1000,
        |user, _now, _users| async move { user.lock().await.ocid_auth().await.is_ok() },
    )
    .await;
}

#[register_test("Get Account Info", WithUsers)]
pub async fn test_get_info(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "get_info",
        users,
        1000,
        1000,
        |user, now, users| async move {
            let user_idx = (now + 1) % users.len();
            let target_id = users[user_idx].lock().await.id;
            user.lock()
                .await
                .oc()
                .get_account_info(GetAccountInfoRequest {
                    id: Some(target_id.0),
                    request_values: vec![],
                })
                .await
                .is_ok()
        },
    )
    .await;
}

#[register_test("Set Account Info", WithUsers)]
pub async fn test_set_account_info(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "set_account_info",
        users,
        1000,
        1000,
        |user, _now, _users| async move {
            user.lock()
                .await
                .oc()
                .set_self_info(SetSelfInfoRequest {
                    user_name: Some(format!("user_{}", rand::random::<u32>())),
                    ..Default::default()
                })
                .await
                .is_ok()
        },
    )
    .await;
}

#[register_test("Unregister Users", WithUsers)]
pub async fn test_unregister(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "unregister",
        users,
        1000,
        1000,
        |user, _now, _users| async move { user.lock().await.unregister().await.is_ok() },
    )
    .await;
}
