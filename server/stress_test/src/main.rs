#![feature(duration_constructors_lite)]

mod framework;

use base::consts::{CONFIG_FILE_ENV_VAR, OCID};
use clap::Parser;
use client::helper;
use dashmap::DashMap;
use framework::{Output, Record, Report, StressTest};
use pb::service::basic::v1::{GetServerInfoRequest, TimestampRequest};
use pb::{self};
use size::Size;
use std::{
    env::set_var,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn test_timestamp(stress_test: &mut StressTest, app: &mut client::TestApp) -> Output {
    let app = app.clients.clone();
    stress_test
        .stress_test(move || {
            let mut app = app.clone();
            async move { app.basic.timestamp(TimestampRequest {}).await.is_ok() }
        })
        .await
}

async fn test_get_server_into(stress_test: &mut StressTest, app: &mut client::TestApp) -> Output {
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

type UsersGroup = Vec<Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>>;

async fn test_register(report: &mut Report, app: &mut client::TestApp) -> UsersGroup {
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let mut users = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let tmp = Arc::new(tokio::sync::Mutex::new(
            client::oc_helper::user::TestUser::random(app).await,
        ));
        users.push(tmp);
    }
    let idx = Arc::new(AtomicUsize::new(0));
    let users = users;
    let users_test = users.clone();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::SeqCst);
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

    app.owned_users.extend(users_test.clone().into_iter());
    report.add_record(Record::new("register", output));
    users_test
}

async fn test_auth(users: &UsersGroup, report: &mut Report) {
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
    use pb::service::ourchat::get_account_info::v1::*;
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
                user.lock()
                    .await
                    .oc()
                    .get_account_info(GetAccountInfoRequest {
                        id: None,
                        request_values: vec![
                            QueryValues::Ocid.into(),
                            QueryValues::UserName.into(),
                            QueryValues::Email.into(),
                            QueryValues::Friends.into(),
                            QueryValues::UpdatedTime.into(),
                            QueryValues::RegisterTime.into(),
                            QueryValues::PublicUpdatedTime.into(),
                        ],
                    })
                    .await
                    .is_ok()
            }
        })
        .await;
    report.add_record(Record::new("get_info", output));
}

async fn test_upload(
    users: &UsersGroup,
    report: &mut Report,
) -> anyhow::Result<Arc<DashMap<OCID, String>>> {
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let file = helper::generate_file(Size::from_mebibytes(1))?;
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
                match user.lock().await.post_file_as_iter(content).await {
                    Ok(key) => {
                        keys.insert(user_id, key);
                        true
                    }
                    Err(_) => false,
                }
            }
        })
        .await;
    report.add_record(Record::new("upload", output));
    Ok(keys_ret)
}

async fn test_download(keys: Arc<DashMap<OCID, String>>, users: &UsersGroup, report: &mut Report) {
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

async fn test_endpoint(app: &mut client::TestApp) -> anyhow::Result<()> {
    let mut report = Report::new();
    test_basic_service(&mut report, app).await;
    let users = test_register(&mut report, app).await;
    test_auth(&users, &mut report).await;
    test_get_info(&users, &mut report).await;
    let keys = test_upload(&users, &mut report).await?;
    test_download(keys.clone(), &users, &mut report).await;
    println!("{report}");
    Ok(())
}

#[derive(Debug, Parser, Default)]
#[command(author = "SkyUOI", about = "The Stress Test of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "The path of server config")]
    pub config: Option<String>,
    #[arg(
        short,
        long,
        help = "Whether to use existing instance",
        default_value_t = false
    )]
    pub use_exists_instance: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = ArgsParser::parse();
    let mut app = if args.use_exists_instance {
        base::log::logger_init(true, None, std::io::stdout, "ourchat");
        let cfg = server::get_configuration(args.config.iter().map(PathBuf::from).collect())?;
        client::TestApp::new_with_existing_instance(cfg).await?
    } else {
        if let Some(path) = args.config {
            unsafe {
                set_var(CONFIG_FILE_ENV_VAR, path);
            }
        }
        client::TestApp::new_with_launching_instance().await?
    };
    // test every endpoint's performance
    test_endpoint(&mut app).await?;
    app.async_drop().await;
    Ok(())
}
