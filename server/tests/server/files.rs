use claims::assert_err;
use client::TestApp;
use client::helper::{generate_file, get_hash_from_download, get_hash_from_file};
use size::Size;

async fn download(
    app: &mut TestApp,
    small_file: impl Iterator<Item = Vec<u8>> + Clone,
    small_file_hash: &str,
) {
    let user1 = app.new_user().await.unwrap();

    let key = user1
        .lock()
        .await
        .post_file_as_iter(small_file.clone())
        .await
        .unwrap();
    // Allow
    let file_download = user1.lock().await.download_file_as_iter(key).await.unwrap();

    assert_eq!(
        get_hash_from_download(file_download).await.unwrap(),
        small_file_hash
    );
    // Deny
    let user2 = app.new_user().await.unwrap();
    let key2 = user2
        .lock()
        .await
        .post_file_as_iter(small_file)
        .await
        .unwrap();
    assert_err!(user1.lock().await.download_file(key2).await);
}

async fn clean_files(
    app: &mut TestApp,
    small_file: impl Iterator<Item = Vec<u8>> + Clone,
    small_file_hash: &str,
    max_size: Size,
) {
    let user = app.new_user().await.unwrap();

    let big_file_size = Size::from_mebibytes(1.5);
    let big_file = generate_file(big_file_size).unwrap();

    let mut key = Vec::new();
    for i in 0..max_size.bytes() / big_file_size.bytes() {
        key.push(
            user.lock()
                .await
                .post_file_as_iter(big_file.clone())
                .await
                .unwrap(),
        );
        tracing::debug!("Uploaded File {}", i);
    }
    tracing::debug!(
        "Limit Size: {} bytes / Per File's Size {} bytes ({} files)",
        max_size,
        big_file_size,
        max_size.bytes() / big_file_size.bytes()
    );
    let file_key = user
        .lock()
        .await
        .post_file_as_iter(small_file.clone())
        .await
        .unwrap();
    tracing::debug!("small file key: {}", &file_key);
    let file_download = user
        .lock()
        .await
        .download_file_as_iter(file_key)
        .await
        .unwrap();
    for (id, key) in key.iter().enumerate() {
        tracing::debug!("file_key {id}: {}", &key);
    }
    assert_err!(user.lock().await.download_file(key[0].clone()).await);
    assert_eq!(
        get_hash_from_download(file_download).await.unwrap(),
        small_file_hash
    );
}

async fn deny_too_big_file(app: &mut TestApp) {
    let user = app.new_user().await.unwrap();
    let super_big_file = generate_file(Size::from_mebibytes(20)).unwrap();
    assert_err!(user.lock().await.post_file_as_iter(super_big_file).await);
}

#[tokio::test]
async fn upload() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    let user_files_limit = Size::from_mebibytes(10);
    config.main_cfg.user_files_limit = user_files_limit;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args))
        .await
        .unwrap();
    let small_file = generate_file(Size::from_mebibytes(1.5)).unwrap();
    let small_file_hash = get_hash_from_file(small_file.clone());
    download(&mut app, small_file.clone(), &small_file_hash).await;
    clean_files(&mut app, small_file, &small_file_hash, user_files_limit).await;
    deny_too_big_file(&mut app).await;
    app.async_drop().await;
}
