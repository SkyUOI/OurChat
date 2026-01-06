use crate::UsersGroup;
use crate::framework::{Record, Report, StressTest};
use base::consts::OCID;
use client::helper;
use dashmap::DashMap;
use size::Size;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use derive::register_test;

#[register_test("File Upload", WithUsers)]
pub async fn test_upload(
    users: &UsersGroup,
    report: &mut Report,
) -> anyhow::Result<Arc<DashMap<OCID, String>>> {
    tracing::info!("▶️  Running test: 'upload'");
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
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let content = file.clone();
            let keys = keys.clone();
            async move {
                // Get user_id in one lock, then make the call in another to avoid deadlock
                let user_id = {
                    let u = user.lock().await;
                    u.ocid.clone()
                };
                match user.lock().await.post_file_as_iter(content, None).await {
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

#[register_test("File Download", WithUsers)]
pub async fn test_download(
    keys: Arc<DashMap<OCID, String>>,
    users: &UsersGroup,
    report: &mut Report,
) {
    tracing::info!("▶️  Running test: 'download'");
    let mut stress_test = StressTest::builder()
        .set_concurrency(1000)
        .set_requests(1000);
    let users = users.clone();
    let idx = Arc::new(AtomicUsize::new(0));
    let output = stress_test
        .stress_test(move || {
            let now = idx.fetch_add(1, Ordering::Relaxed);
            let user = users[now].clone();
            let keys = keys.clone();
            async move {
                // Get key in one lock, then make the call in another to avoid deadlock
                let key = {
                    let k = keys.get(&user.lock().await.ocid);
                    k.map(|k| k.clone())
                };
                if let Some(key) = key {
                    user.lock().await.download_file(key).await.is_ok()
                } else {
                    false
                }
            }
        })
        .await;
    report.add_record(Record::new("download", output));
}
