#![feature(duration_constructors)]

mod framework;

use clap::Parser;
use dashmap::DashMap;
use framework::{Output, Record, Report, StressTest};
use parking_lot::Mutex;
use server::{
    consts::ID,
    pb::{self, get_info::GetAccountInfoRequest, upload::UploadRequest},
};
use std::{
    env::set_var,
    fmt,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{fs::read_to_string, time::Instant};
use tonic::IntoRequest;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn test_timestamp(stress_test: &mut StressTest, app: &mut client::TestApp) -> Output {
    let app = app.clients.clone();
    stress_test
        .stress_test(move || {
            let mut app = app.clone();
            async move { app.basic.timestamp(()).await.is_ok() }
        })
        .await
}

async fn test_get_server_into(stress_test: &mut StressTest, app: &mut client::TestApp) -> Output {
    let app = app.clients.clone();
    stress_test
        .stress_test(move || {
            let mut app = app.clone();
            async move { app.basic.get_server_info(()).await.is_ok() }
        })
        .await
}

async fn test_basic_service(report: &mut Report, app: &mut client::TestApp) {
    let mut stress_test = StressTest::builder()
        .set_concurrency(100)
        .set_requests(1000);
    report.add_record(Record::new(
        "timestamp",
        test_timestamp(&mut stress_test, app).await,
    ));
    report.add_record(Record::new(
        "get_server_info",
        test_get_server_into(&mut stress_test, app).await,
    ));
}

type UsersGroup = Vec<Arc<tokio::sync::Mutex<client::TestUser>>>;

async fn test_register(report: &mut Report, app: &mut client::TestApp) -> UsersGroup {
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let mut users = Vec::with_capacity(1000);
    for _ in 0..1000 {
        users.push(Arc::new(tokio::sync::Mutex::new(
            client::TestUser::random(app).await,
        )));
    }
    let idx = Arc::new(AtomicUsize::new(0));
    let users = users;
    let users_test = users.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::SeqCst);
            let user = users[now].clone();
            async move { user.lock().await.register().await.is_ok() }
        })
        .await;

    app.owned_users.extend(users_test.clone().into_iter());
    report.add_record(Record::new("register", output));
    users_test
}

async fn test_auth(users: &UsersGroup, report: &mut Report, app: &mut client::TestApp) {
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::SeqCst);
            let user = users[now].clone();
            async move { user.lock().await.ocid_auth().await.is_ok() }
        })
        .await;
    report.add_record(Record::new("auth", output));
}

async fn test_get_info(users: &UsersGroup, report: &mut Report) {
    use pb::get_info::*;
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::SeqCst);
            let user = users[now].clone();
            async move {
                let ocid = user.lock().await.ocid.clone();
                user.lock()
                    .await
                    .oc()
                    .get_info(GetAccountInfoRequest {
                        ocid: ocid.clone(),
                        request_values: vec![
                            RequestValues::Ocid.into(),
                            RequestValues::UserName.into(),
                            RequestValues::Email.into(),
                            RequestValues::Friends.into(),
                            RequestValues::UpdateTime.into(),
                            RequestValues::RegisterTime.into(),
                            RequestValues::PublicUpdateTime.into(),
                        ],
                    })
                    .await
                    .is_ok()
            }
        })
        .await;
    report.add_record(Record::new("get_info", output));
}

async fn test_upload(users: &UsersGroup, report: &mut Report) -> Arc<DashMap<String, String>> {
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let file = read_to_string("server/tests/server/test_data/big_file.txt")
        .await
        .unwrap();
    let keys = Arc::new(DashMap::new());
    let keys_ret = keys.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::SeqCst);
            let user = users[now].clone();
            let content = file.clone();
            let keys = keys.clone();
            async move {
                let user_id = user.lock().await.ocid.clone();
                match user.lock().await.post_file(content).await {
                    Ok(key) => {
                        keys.insert(user_id, key);
                        true
                    }
                    Err(e) => false,
                }
            }
        })
        .await;
    report.add_record(Record::new("upload", output));
    keys_ret
}

async fn test_download(
    keys: Arc<DashMap<String, String>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::SeqCst);
            let user = users[now].clone();
            let keys = keys.clone();
            async move {
                let key = keys.get(&user.lock().await.ocid).unwrap();
                user.lock().await.download_file(key.clone()).await.is_ok()
            }
        })
        .await;
    report.add_record(Record::new("download", output));
}

async fn test_endpoint(app: &mut client::TestApp) {
    let mut report = Report::new();
    test_basic_service(&mut report, app).await;
    let users = test_register(&mut report, app).await;
    test_auth(&users, &mut report, app).await;
    test_get_info(&users, &mut report).await;
    let keys = test_upload(&users, &mut report).await;
    test_download(keys.clone(), &users, &mut report).await;
    println!("{}", report);
}

#[derive(Debug, Parser, Default)]
#[command(author = "SkyUOI", about = "The Stress Test of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "The path of server config")]
    pub config: Option<String>,
    #[arg(
        long,
        help = "Whether to use existing instance",
        default_value_t = false
    )]
    pub use_exists_instance: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = ArgsParser::parse();
    let mut app = if args.use_exists_instance {
        let cfg = server::get_configuration(args.config.as_ref().map(PathBuf::from))?;
        client::TestApp::new_with_existing_instance(cfg)
            .await
            .unwrap()
    } else {
        if let Some(path) = args.config {
            unsafe {
                set_var("OURCHAT_CONFIG_FILE", path);
            }
        }
        client::TestApp::new_with_launching_instance(None)
            .await
            .unwrap()
    };
    // test every endpoint's performance
    test_endpoint(&mut app).await;
    app.async_drop().await;
    Ok(())
}
