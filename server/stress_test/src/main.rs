#![feature(duration_constructors)]

use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn test_timestamp(app: &mut client::TestApp) {
    let max_time = Arc::new(Mutex::new(Duration::from_nanos(0)));
    let min_time = Arc::new(Mutex::new(Duration::from_hours(1)));
    let barrier = Arc::new(tokio::sync::Barrier::new(150 + 1));
    let total = Instant::now();
    for _ in 0..150 {
        let mut clients = app.clients.clone();
        let barrier = barrier.clone();
        let max_time = max_time.clone();
        let min_time = min_time.clone();
        tokio::spawn(async move {
            for _ in 0..5 {
                let instant = Instant::now();
                clients.basic.timestamp(()).await.unwrap();
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
    println!(
        "total: {:?}, max_time: {:?}, min_time: {:?}, QPS: {}",
        total.elapsed(),
        *max_time.lock(),
        *min_time.lock(),
        (150 * 5) as f64 / total.elapsed().as_secs_f64()
    );
}

async fn test_basic_service(app: &mut client::TestApp) {
    test_timestamp(app).await;
}

async fn test_endpoint(app: &mut client::TestApp) {
    test_basic_service(app).await;
}

#[tokio::main]
async fn main() {
    let mut app = client::TestApp::new(None).await.unwrap();
    // test every endpoint's performance
    test_endpoint(&mut app).await;
    app.async_drop().await;
}
