use std::{
    fs::{self, File},
    path::PathBuf,
    time::Duration,
};

use base::consts::{HTTP_SERVER_LOG_PREFIX, LOG_OUTPUT_DIR};
use chrono::Datelike;
use client::TestHttpApp;

#[tokio::test]
async fn test_log_clear() {
    let mut config = TestHttpApp::get_config().await.unwrap();
    config.main_cfg.log_clean_duration = Duration::from_secs(1);

    fs::create_dir_all(LOG_OUTPUT_DIR).unwrap();
    let path = PathBuf::from(LOG_OUTPUT_DIR).join(PathBuf::from(format!(
        "{HTTP_SERVER_LOG_PREFIX}.2020-01-01"
    )));
    File::create(&path).unwrap();
    let now = chrono::Local::now().date_naive();
    let path2 = PathBuf::from(LOG_OUTPUT_DIR).join(PathBuf::from(format!(
        "{HTTP_SERVER_LOG_PREFIX}.{}-{}-{}",
        now.year() + 1,
        now.month(),
        now.day()
    )));
    File::create(&path2).unwrap();
    let mut http_app = TestHttpApp::new_custom(config, None).await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(!path.exists());
    assert!(path2.exists());
    fs::remove_file(&path2).unwrap();

    http_app.async_drop().await;
}
