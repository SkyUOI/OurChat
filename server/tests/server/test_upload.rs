use claims::assert_err;
use server::consts::{Bt, FileSize, MBt};
use tokio::fs::read_to_string;

#[tokio::test]
async fn test_upload() {
    let (mut config, args) = client::TestApp::get_test_config().unwrap();
    config.main_cfg.user_files_limit = FileSize::MB(MBt(10));
    let mut app = client::TestApp::new_with_launching_instance_custom_cfg(None, (config, args))
        .await
        .unwrap();
    let user1 = app.new_user().await.unwrap();

    let file = read_to_string("tests/server/test_data/file1.txt")
        .await
        .unwrap();
    let key = user1.lock().await.post_file(file.clone()).await.unwrap();
    // Allow
    let file_download = user1.lock().await.download_file(key).await.unwrap();
    assert_eq!(&file_download, file.as_bytes());
    // Deny
    let user2 = app.new_user().await.unwrap();
    let key2 = user2.lock().await.post_file(file.clone()).await.unwrap();
    claims::assert_err!(user1.lock().await.download_file(key2).await);

    // Test cleaning files
    let user3 = app.new_user().await.unwrap();

    let big_file = read_to_string("tests/server/test_data/big_file.txt")
        .await
        .unwrap();

    let max_size: Bt = app.user_files_limit.into();
    let max_size: u64 = *max_size;
    let big_file_size = big_file.len() as u64;

    let mut key = Vec::new();
    for _ in 0..max_size / big_file_size {
        key.push(
            user3
                .lock()
                .await
                .post_file(big_file.clone())
                .await
                .unwrap(),
        );
    }
    tracing::debug!(
        "Limit Size: {} bytes / Per File's Size {} bytes ({} files)",
        max_size,
        big_file_size,
        max_size / big_file_size
    );
    user3
        .lock()
        .await
        .post_file(big_file.clone())
        .await
        .unwrap();
    assert_err!(user3.lock().await.download_file(key[0].clone()).await);

    // for i in {

    // }

    app.async_drop().await;
}
