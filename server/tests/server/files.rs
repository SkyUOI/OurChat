use claims::assert_err;
use client::TestApp;
use client::helper::{generate_file, get_hash_from_download, get_hash_from_file};
use client::oc_helper::user::TestUserShared;
use size::Size;
use tokio::fs;

/// Helper function to download a file and verify its hash
async fn verify_file_download(
    user: &TestUserShared,
    key: String,
    expected_hash: &str,
) -> anyhow::Result<()> {
    let file_download = user.lock().await.download_file_as_iter(key).await?;
    assert_eq!(
        get_hash_from_download(file_download).await.unwrap(),
        expected_hash
    );
    Ok(())
}

/// Helper function to check if temporary files remain
async fn no_temp_files_remain(app: &TestApp) -> bool {
    let files_storage_path = &app.app_shared.cfg.main_cfg.files_storage_path;
    match fs::read_dir(files_storage_path).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await
                    && file_type.is_file()
                    && entry.path().extension() == Some("tmp".as_ref())
                {
                    tracing::warn!("Found temporary file: {}", entry.path().display());
                    return false;
                }
            }
            true
        }
        Err(_) => true, // If directory doesn't exist, no temp files
    }
}

async fn test_upload_and_download(
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
    verify_file_download(&user1, key, small_file_hash)
        .await
        .unwrap();

    let user2 = app.new_user().await.unwrap();
    let key2 = user2
        .lock()
        .await
        .post_file_as_iter(small_file)
        .await
        .unwrap();
    assert_err!(user1.lock().await.download_file(key2).await);
}

async fn test_files_upload_overflow_and_delete(
    app: &mut TestApp,
    small_file: impl Iterator<Item = Vec<u8>> + Clone,
    small_file_hash: &str,
    max_size: Size,
) {
    let user = app.new_user().await.unwrap();
    let big_file_size = Size::from_mebibytes(1.5);
    let big_file = generate_file(big_file_size).unwrap();

    let mut keys = Vec::new();
    for _ in 0..max_size.bytes() / big_file_size.bytes() {
        keys.push(
            user.lock()
                .await
                .post_file_as_iter(big_file.clone())
                .await
                .unwrap(),
        );
    }

    let small_file_key = user
        .lock()
        .await
        .post_file_as_iter(small_file.clone())
        .await
        .unwrap();
    assert_err!(user.lock().await.download_file(keys[0].clone()).await);
    verify_file_download(&user, small_file_key, small_file_hash)
        .await
        .unwrap();
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
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    let small_file = generate_file(Size::from_mebibytes(1.5)).unwrap();
    let small_file_hash = get_hash_from_file(small_file.clone());

    test_upload_and_download(&mut app, small_file.clone(), &small_file_hash).await;
    assert!(no_temp_files_remain(&app).await);

    test_files_upload_overflow_and_delete(
        &mut app,
        small_file.clone(),
        &small_file_hash,
        user_files_limit,
    )
    .await;
    assert!(no_temp_files_remain(&app).await);

    deny_too_big_file(&mut app).await;
    assert!(no_temp_files_remain(&app).await);

    app.async_drop().await;
}
