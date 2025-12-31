use std::{
    fmt,
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use parking_lot::Mutex;
use tokio::time::Instant;

/// Re-export commonly used types for convenience
pub use std::sync::atomic::Ordering;

pub struct StressTest {
    concurrency: usize,
    requests: usize,
}

pub struct Record {
    name: String,
    record: Output,
}

impl Record {
    pub fn new(name: impl Into<String>, record: Output) -> Self {
        Self {
            name: name.into(),
            record,
        }
    }
}

pub struct Report {
    records: Vec<Record>,
}

impl Report {
    pub fn new() -> Self {
        Self { records: vec![] }
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn generate_output(&self) -> String {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL).set_header(vec![
            "Test", "Status", "Success", "Failed", "QPS", "Time(ms)",
        ]);

        let mut total_success = 0;
        let mut total_failed = 0;

        for record in &self.records {
            let status = if record.record.failed > 0 {
                Cell::new("❌ FAIL")
                    .fg(Color::Red)
                    .add_attribute(Attribute::Bold)
            } else {
                Cell::new("✅ PASS")
                    .fg(Color::Green)
                    .add_attribute(Attribute::Bold)
            };

            table.add_row(vec![
                Cell::new(&record.name),
                status,
                Cell::new(record.record.success.to_string()),
                Cell::new(record.record.failed.to_string()),
                Cell::new(format!("{:.1}", record.record.qps)),
                Cell::new(format!("{:.0}", record.record.total.as_secs_f64() * 1000.0)),
            ]);

            total_success += record.record.success;
            total_failed += record.record.failed;
        }

        // Calculate overall statistics
        let total = total_success + total_failed;
        let success_rate = if total > 0 {
            (total_success as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        format!("\n{table}\n\nTotal: {total_success}/{total} tests passed ({success_rate:.1}%)\n")
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_output())
    }
}

#[derive(Debug)]
pub struct Output {
    success: usize,
    failed: usize,
    max_time: Duration,
    min_time: Duration,
    qps: f64,
    total: Duration,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.success + self.failed;
        let success_rate = if total > 0 {
            (self.success as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        write!(
            f,
            "📊 Results:\n\
             ✅ Success: {} ({:.1}%)\n\
             ❌ Failed: {} ({:.1}%)\n\
             ⏱️  Max Time: {:.2}ms\n\
             ⏱️  Min Time: {:.2}ms\n\
             📈 QPS: {:.2}\n\
             🕐 Total: {:.2}s\n",
            self.success,
            success_rate,
            self.failed,
            100.0 - success_rate,
            self.max_time.as_secs_f64() * 1000.0,
            self.min_time.as_secs_f64() * 1000.0,
            self.qps,
            self.total.as_secs_f64()
        )
    }
}

impl StressTest {
    pub fn builder() -> Self {
        Self {
            concurrency: 50,
            requests: 200,
        }
    }

    pub fn set_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    pub fn set_requests(mut self, requests: usize) -> Self {
        self.requests = requests;
        self
    }

    pub async fn stress_test<F, R>(&mut self, logic: F) -> Output
    where
        F: FnMut() -> R + Send + Sync + 'static + Clone,
        R: Future<Output = bool> + Send + 'static,
    {
        tracing::info!(
            "🚀 Starting stress test: concurrency={}, requests={}",
            self.concurrency,
            self.requests
        );
        let max_time = Arc::new(Mutex::new(Duration::from_nanos(0)));
        let min_time = Arc::new(Mutex::new(Duration::from_hours(1)));
        let failed = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(tokio::sync::Barrier::new(self.concurrency + 1));
        let correct = Arc::new(AtomicUsize::new(0));
        let total = Instant::now();
        let requests_per_concurrency = self.requests / self.concurrency;
        tracing::debug!(
            "Requests per concurrency worker: {}",
            requests_per_concurrency
        );
        for _ in 0..self.concurrency {
            let barrier = barrier.clone();
            let max_time = max_time.clone();
            let min_time = min_time.clone();
            let correct = correct.clone();
            let failed = failed.clone();
            let mut logic = logic.clone();
            tokio::spawn(async move {
                for i in 0..requests_per_concurrency {
                    let instant = Instant::now();
                    let success = logic().await;
                    let elapsed = instant.elapsed();
                    if success {
                        correct.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    } else {
                        failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tracing::trace!("Request failed (worker iteration {})", i);
                    }
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
        let failed_count = failed.load(std::sync::atomic::Ordering::Relaxed);
        let elapsed = total.elapsed();

        tracing::info!(
            "✅ Stress test completed: success={}, failed={}, total_time={:.2}s, qps={:.2}",
            success,
            failed_count,
            elapsed.as_secs_f64(),
            self.requests as f64 / elapsed.as_secs_f64()
        );

        Output {
            max_time: *max_time.lock(),
            min_time: *min_time.lock(),
            qps: self.requests as f64 / elapsed.as_secs_f64(),
            success,
            failed: self.requests - success,
            total: elapsed,
        }
    }
}

/// Helper function to run a stress test with a user-indexed operation
/// Reduces boilerplate in test functions
pub async fn run_user_stress_test<F, R>(
    report: &mut Report,
    test_name: &str,
    users: &[Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>],
    concurrency: usize,
    requests: usize,
    logic: F,
) where
    F: FnMut(
            Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>,
            usize,
            Vec<Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>>,
        ) -> R
        + Send
        + Sync
        + 'static
        + Clone,
    R: Future<Output = bool> + Send + 'static,
{
    tracing::info!(
        "▶️  Running test: '{}' with {} users",
        test_name,
        users.len()
    );
    let mut stress_test = StressTest::builder()
        .set_concurrency(concurrency)
        .set_requests(requests);
    let idx = Arc::new(AtomicUsize::new(0));
    let users = users.to_vec();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now % users.len()].clone();
            let users = users.clone();
            let mut logic = logic.clone();
            async move { logic(user, now, users).await }
        })
        .await;

    // Log summary after test completes
    if output.failed > 0 {
        tracing::warn!(
            "⚠️  Test '{}': {}/{} requests failed ({:.1}%)",
            test_name,
            output.failed,
            output.success + output.failed,
            (output.failed as f64 / (output.success + output.failed) as f64) * 100.0
        );
    } else {
        tracing::info!(
            "✅ Test '{}' passed: {} successful requests",
            test_name,
            output.success
        );
    }

    report.add_record(Record::new(test_name, output));
}

/// Helper function for session tests that need both users and sessions
pub async fn run_session_stress_test<F, R>(
    report: &mut Report,
    test_name: &str,
    sessions: Arc<dashmap::DashMap<base::consts::ID, base::consts::SessionID>>,
    users: &[Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>],
    concurrency: usize,
    requests: usize,
    logic: F,
) where
    F: FnMut(
            Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>,
            usize,
            Vec<Arc<tokio::sync::Mutex<client::oc_helper::user::TestUser>>>,
            Arc<dashmap::DashMap<base::consts::ID, base::consts::SessionID>>,
        ) -> R
        + Send
        + Sync
        + 'static
        + Clone,
    R: Future<Output = bool> + Send + 'static,
{
    tracing::info!(
        "▶️  Running test: '{}' with {} users",
        test_name,
        users.len()
    );
    let mut stress_test = StressTest::builder()
        .set_concurrency(concurrency)
        .set_requests(requests);
    let idx = Arc::new(AtomicUsize::new(0));
    let users = users.to_vec();
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now % users.len()].clone();
            let users = users.clone();
            let sessions = sessions.clone();
            let mut logic = logic.clone();
            async move { logic(user, now, users, sessions).await }
        })
        .await;

    // Log summary after test completes
    if output.failed > 0 {
        tracing::warn!(
            "⚠️  Test '{}': {}/{} requests failed ({:.1}%)",
            test_name,
            output.failed,
            output.success + output.failed,
            (output.failed as f64 / (output.success + output.failed) as f64) * 100.0
        );
    } else {
        tracing::info!(
            "✅ Test '{}' passed: {} successful requests",
            test_name,
            output.success
        );
    }

    report.add_record(Record::new(test_name, output));
}
