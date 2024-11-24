#![feature(duration_constructors)]

use parking_lot::Mutex;
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

struct StressTest {
    concurrency: usize,
    requests: usize,
}

struct Record {
    name: String,
    record: Output,
}

struct Report {
    records: Vec<Record>,
}

impl Report {
    fn new() -> Self {
        Self { records: vec![] }
    }

    fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    fn generate_output(&self) -> String {
        let mut s = String::new();
        for record in &self.records {
            s.push_str(&"-".repeat(100));
            s.push_str(&format!(
                "\nRpc {}:\n\n\n{}\n\n",
                record.name, record.record
            ));
        }
        s
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_output())
    }
}

#[derive(Debug)]
struct Output {
    success: usize,
    failed: usize,
    max_time: Duration,
    min_time: Duration,
    qps: f64,
    total: Duration,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Summary:\nSuccess: {}\nFailed: {}\nMax Time: {:?}\nMin Time: {:?}\nQPS: {}\nTotal: {:?}\n",
            self.success, self.failed, self.max_time, self.min_time, self.qps, self.total
        )
    }
}

impl StressTest {
    fn builder() -> Self {
        Self {
            concurrency: 50,
            requests: 200,
        }
    }

    fn set_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    fn set_requests(mut self, requests: usize) -> Self {
        self.requests = requests;
        self
    }

    async fn stress_test<F, R>(&mut self, logic: F) -> Output
    where
        F: FnMut() -> R + Send + Sync + 'static + Clone,
        R: Future<Output = bool> + Send + 'static,
    {
        let max_time = Arc::new(Mutex::new(Duration::from_nanos(0)));
        let min_time = Arc::new(Mutex::new(Duration::from_hours(1)));
        let barrier = Arc::new(tokio::sync::Barrier::new(self.concurrency + 1));
        let correct = Arc::new(AtomicUsize::new(0));
        let total = Instant::now();
        let requests_per_concurrency = self.requests / self.concurrency;
        for _ in 0..self.concurrency {
            let barrier = barrier.clone();
            let max_time = max_time.clone();
            let min_time = min_time.clone();
            let correct = correct.clone();
            let mut logic = logic.clone();
            tokio::spawn(async move {
                for _ in 0..requests_per_concurrency {
                    let instant = Instant::now();
                    let success = logic().await;
                    if success {
                        correct.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                    let elapsed = instant.elapsed();
                    if elapsed > *max_time.lock() {
                        *max_time.lock() = elapsed;
                    }
                    if elapsed < *min_time.lock() {
                        *min_time.lock() = elapsed;
                    }
                }
                barrier.wait().await;
            });
        }
        barrier.wait().await;
        let success = correct.load(std::sync::atomic::Ordering::Relaxed);
        Output {
            max_time: *max_time.lock(),
            min_time: *min_time.lock(),
            qps: self.requests as f64 / total.elapsed().as_secs_f64(),
            success,
            failed: self.requests - success,
            total: total.elapsed(),
        }
    }
}

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
    report.add_record(Record {
        name: "timestamp".to_string(),
        record: test_timestamp(&mut stress_test, app).await,
    });
    report.add_record(Record {
        name: "get_server_info".to_string(),
        record: test_get_server_into(&mut stress_test, app).await,
    });
}

async fn test_register(
    report: &mut Report,
    app: &mut client::TestApp,
) -> Vec<Arc<tokio::sync::Mutex<client::TestUser>>> {
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
    report.add_record(Record {
        name: "register".to_string(),
        record: output,
    });
    users_test
}

async fn test_endpoint(app: &mut client::TestApp) {
    let mut report = Report::new();
    test_basic_service(&mut report, app).await;
    test_register(&mut report, app).await;
    println!("{}", report);
}

#[tokio::main]
async fn main() {
    let mut app = client::TestApp::new(None).await.unwrap();
    // test every endpoint's performance
    test_endpoint(&mut app).await;
    app.async_drop().await;
}
