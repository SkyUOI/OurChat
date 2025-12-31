use crate::UsersGroup;
use crate::framework::{Output, Record, Report, StressTest, run_user_stress_test};
use pb::service::basic::preset_user_status::v1::GetPresetUserStatusRequest;
use pb::service::basic::v1::{GetIdRequest, GetServerInfoRequest, TimestampRequest};

pub async fn test_timestamp(stress_test: &mut StressTest, app: &mut client::ClientCore) -> Output {
    let app = app.clients.clone();
    stress_test
        .stress_test(move || {
            let mut app = app.clone();
            async move { app.basic.timestamp(TimestampRequest {}).await.is_ok() }
        })
        .await
}

pub async fn test_get_server_info(
    stress_test: &mut StressTest,
    app: &mut client::ClientCore,
) -> Output {
    let app = app.clients.clone();
    stress_test
        .stress_test(move || {
            let mut app = app.clone();
            async move {
                app.basic
                    .get_server_info(GetServerInfoRequest {})
                    .await
                    .is_ok()
            }
        })
        .await
}

pub async fn test_get_id(users: &UsersGroup, report: &mut Report) {
    run_user_stress_test(
        report,
        "get_id",
        users,
        1000,
        1000,
        |user, _now, _users| async move {
            // Get both clients and ocid in a single lock to avoid deadlock
            let (mut clients, ocid) = {
                let user = user.lock().await;
                (user.clients.clone(), user.ocid.0.clone())
            };
            clients.basic.get_id(GetIdRequest { ocid }).await.is_ok()
        },
    )
    .await;
}

pub async fn test_preset_user_status(report: &mut Report, app: &mut client::ClientCore) {
    tracing::info!("▶️  Running test: 'preset_user_status'");
    let app = app.clients.clone();
    let mut stress_test = StressTest::builder()
        .set_concurrency(100)
        .set_requests(1000);
    let output = stress_test
        .stress_test(move || {
            let mut app = app.clone();
            async move {
                app.basic
                    .get_preset_user_status(GetPresetUserStatusRequest {})
                    .await
                    .is_ok()
            }
        })
        .await;
    report.add_record(Record::new("preset_user_status", output));
}

pub async fn test_basic_service(report: &mut Report, app: &mut client::ClientCore) {
    tracing::info!("▶️  Running test: 'timestamp'");
    let mut stress_test = StressTest::builder()
        .set_concurrency(100)
        .set_requests(1000);
    report.add_record(Record::new(
        "timestamp",
        test_timestamp(&mut stress_test, app).await,
    ));

    tracing::info!("▶️  Running test: 'get_server_info'");
    let mut stress_test = StressTest::builder()
        .set_concurrency(100)
        .set_requests(1000);
    report.add_record(Record::new(
        "get_server_info",
        test_get_server_info(&mut stress_test, app).await,
    ));
}
