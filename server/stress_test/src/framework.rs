use std::{
    fmt,
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};

use parking_lot::Mutex;
use tokio::time::Instant;

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
        write!(
            f,
            "Summary:\nSuccess: {}\nFailed: {}\nMax Time: {:?}\nMin Time: {:?}\nQPS: {}\nTotal: {:?}\n",
            self.success, self.failed, self.max_time, self.min_time, self.qps, self.total
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
