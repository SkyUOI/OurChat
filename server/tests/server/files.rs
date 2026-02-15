use bytes::Bytes;
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
    let files_storage_path = app.app_shared.cfg().main_cfg.files_storage_path.clone();
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
        .post_file_as_iter(small_file.clone(), None)
        .await
        .unwrap();
    verify_file_download(&user1, key, small_file_hash)
        .await
        .unwrap();

    let user2 = app.new_user().await.unwrap();
    let key2 = user2
        .lock()
        .await
        .post_file_as_iter(small_file, None)
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
                .post_file_as_iter(big_file.clone(), None)
                .await
                .unwrap(),
        );
    }

    let small_file_key = user
        .lock()
        .await
        .post_file_as_iter(small_file.clone(), None)
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
    assert_err!(
        user.lock()
            .await
            .post_file_as_iter(super_big_file, None)
            .await
    );
}

/// Test session-based file download permissions:
/// - File owner can download their own file
/// - Users in the same session can download the file
/// - Users not in the session cannot download the file
#[tokio::test]
async fn session_file_download_permission() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a session with two users
    let (session_users, session) = app
        .new_session_db_level(2, "test_session", false)
        .await
        .unwrap();
    let user1 = &session_users[0]; // File owner
    let user2 = &session_users[1]; // Session member

    // Create a third user who is NOT in the session
    let user3 = app.new_user().await.unwrap();

    // Create a test file
    let test_file = generate_file(Size::from_kibibytes(100)).unwrap();
    let test_file_hash = get_hash_from_file(test_file.clone());

    // User1 uploads a file associated with the session
    let file_key = user1
        .lock()
        .await
        .post_file_as_iter(test_file.clone(), Some(session.session_id))
        .await
        .unwrap();

    // User1 (owner) should be able to download the file
    verify_file_download(user1, file_key.clone(), &test_file_hash)
        .await
        .unwrap();

    // User2 (session member) should be able to download the file
    verify_file_download(user2, file_key.clone(), &test_file_hash)
        .await
        .unwrap();

    // User3 (NOT in session) should NOT be able to download the file
    assert_err!(user3.lock().await.download_file(file_key.clone()).await);

    app.async_drop().await;
}

/// Test that files without session_id are only accessible to the owner
#[tokio::test]
async fn file_without_session_only_owner_access() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create two users
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();

    // Create a test file
    let test_file = generate_file(Size::from_kibibytes(100)).unwrap();
    let test_file_hash = get_hash_from_file(test_file.clone());

    // User1 uploads a file WITHOUT session_id
    let file_key = user1
        .lock()
        .await
        .post_file_as_iter(test_file.clone(), None)
        .await
        .unwrap();

    // User1 (owner) should be able to download the file
    verify_file_download(&user1, file_key.clone(), &test_file_hash)
        .await
        .unwrap();

    // User2 should NOT be able to download the file (no session association)
    assert_err!(user2.lock().await.download_file(file_key).await);

    app.async_drop().await;
}

/// Test basic file deletion: owner can delete their own file
#[tokio::test]
async fn delete_file_owner() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let user = app.new_user().await.unwrap();
    let test_file = generate_file(Size::from_kibibytes(100)).unwrap();
    let test_file_hash = get_hash_from_file(test_file.clone());

    // Upload a file
    let file_key = user
        .lock()
        .await
        .post_file_as_iter(test_file.clone(), None)
        .await
        .unwrap();

    // Verify file can be downloaded before deletion
    verify_file_download(&user, file_key.clone(), &test_file_hash)
        .await
        .unwrap();

    // Delete the file
    user.lock()
        .await
        .delete_file(file_key.clone())
        .await
        .unwrap();

    // Verify file can no longer be downloaded
    let err = user
        .lock()
        .await
        .download_file(file_key.clone())
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);

    app.async_drop().await;
}

/// Test permission: non-owner cannot delete someone else's file
#[tokio::test]
async fn delete_file_non_owner_permission_denied() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let test_file = generate_file(Size::from_kibibytes(100)).unwrap();

    // User1 uploads a file
    let file_key = user1
        .lock()
        .await
        .post_file_as_iter(test_file, None)
        .await
        .unwrap();

    // User2 (not the owner) should NOT be able to delete the file
    let err = user2
        .lock()
        .await
        .delete_file(file_key.clone())
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);

    // Verify user1 can still download the file
    assert!(user1.lock().await.download_file(file_key).await.is_ok());

    app.async_drop().await;
}

/// Test deleting a non-existent file returns error
#[tokio::test]
async fn delete_nonexistent_file_returns_error() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    let user = app.new_user().await.unwrap();

    // Try to delete a file that doesn't exist
    let fake_key = "nonexistent_file_key_12345";
    let err = user.lock().await.delete_file(fake_key).await.unwrap_err();
    assert_eq!(err.code(), tonic::Code::PermissionDenied);

    app.async_drop().await;
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

/// Test chunked upload API
#[tokio::test]
async fn upload_chunked() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    let user_files_limit = Size::from_mebibytes(10);
    config.main_cfg.user_files_limit = user_files_limit;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    let small_file = generate_file(Size::from_mebibytes(1.5)).unwrap();
    let small_file_hash = get_hash_from_file(small_file.clone());
    let small_file_data: Vec<u8> = small_file.flatten().collect();

    // Test basic chunked upload and download
    let user1 = app.new_user().await.unwrap();
    let key = user1
        .lock()
        .await
        .post_file_chunked(&small_file_data, None)
        .await
        .unwrap();
    verify_file_download(&user1, key, &small_file_hash)
        .await
        .unwrap();

    // Verify no temp files remain
    assert!(no_temp_files_remain(&app).await);

    app.async_drop().await;
}

/// Test chunked upload with session association
#[tokio::test]
async fn upload_chunked_with_session() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a session with two users
    let (session_users, session) = app
        .new_session_db_level(2, "test_session_chunked", false)
        .await
        .unwrap();
    let user1 = &session_users[0]; // File owner
    let user2 = &session_users[1]; // Session member

    // Create a test file
    let test_file = generate_file(Size::from_kibibytes(100)).unwrap();
    let test_file_hash = get_hash_from_file(test_file.clone());
    let test_file_data: Vec<u8> = test_file.flatten().collect();

    // User1 uploads a file using chunked upload API
    let file_key = user1
        .lock()
        .await
        .post_file_chunked(&test_file_data, Some(session.session_id))
        .await
        .unwrap();

    // User1 (owner) should be able to download the file
    verify_file_download(user1, file_key.clone(), &test_file_hash)
        .await
        .unwrap();

    // User2 (session member) should be able to download the file
    verify_file_download(user2, file_key.clone(), &test_file_hash)
        .await
        .unwrap();

    app.async_drop().await;
}

/// Test chunked upload with file size overflow and quota management
#[tokio::test]
async fn upload_chunked_overflow_and_delete() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    let user_files_limit = Size::from_mebibytes(10);
    config.main_cfg.user_files_limit = user_files_limit;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    let small_file = generate_file(Size::from_mebibytes(1.5)).unwrap();
    let small_file_hash = get_hash_from_file(small_file.clone());
    let small_file_data: Vec<u8> = small_file.flatten().collect();
    let big_file_size = Size::from_mebibytes(1.5);
    let big_file = generate_file(big_file_size).unwrap();
    let big_file_data: Vec<u8> = big_file.flatten().collect();

    let user = app.new_user().await.unwrap();

    // Upload multiple big files until quota is exceeded
    let mut keys = Vec::new();
    for _ in 0..user_files_limit.bytes() / big_file_size.bytes() {
        keys.push(
            user.lock()
                .await
                .post_file_chunked(&big_file_data, None)
                .await
                .unwrap(),
        );
    }

    // This upload should trigger cleanup of old files
    let small_file_key = user
        .lock()
        .await
        .post_file_chunked(&small_file_data, None)
        .await
        .unwrap();

    // Oldest file should be deleted
    assert_err!(user.lock().await.download_file(keys[0].clone()).await);

    // New file should be downloadable
    verify_file_download(&user, small_file_key, &small_file_hash)
        .await
        .unwrap();

    // Verify no temp files remain
    assert!(no_temp_files_remain(&app).await);

    app.async_drop().await;
}

/// Test chunked upload size validation
#[tokio::test]
async fn upload_chunked_size_validation() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    let user_files_limit = Size::from_mebibytes(10);
    config.main_cfg.user_files_limit = user_files_limit;
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();

    // Test with a file that's too large
    let super_big_file = generate_file(Size::from_mebibytes(20)).unwrap();
    let super_big_file_data: Vec<u8> = super_big_file.flatten().collect();
    assert_err!(
        user.lock()
            .await
            .post_file_chunked(&super_big_file_data, None)
            .await
    );

    // Verify no temp files remain after failed upload
    assert!(no_temp_files_remain(&app).await);

    app.async_drop().await;
}

/// Test cancel upload functionality
#[tokio::test]
async fn upload_chunked_cancel() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let test_file = generate_file(Size::from_kibibytes(100)).unwrap();
    let test_file_data: Vec<u8> = test_file.flatten().collect();

    // Manually test the cancel flow by simulating a partial upload
    use pb::service::ourchat::upload::v1::{
        CompleteUploadRequest, StartUploadRequest, UploadChunkRequest,
    };
    use sha3::{Digest, Sha3_256};

    let hash = Sha3_256::digest(&test_file_data);
    let size = test_file_data.len() as u64;

    // Start upload session
    let start_response = user
        .lock()
        .await
        .oc()
        .start_upload(StartUploadRequest {
            hash: Bytes::copy_from_slice(&hash),
            size,
            auto_clean: true,
            session_id: None,
        })
        .await
        .unwrap()
        .into_inner();

    let upload_id = start_response.upload_id;

    // Upload first chunk only
    let chunk_size = start_response.chunk_size as usize;
    let chunk = &test_file_data[..chunk_size.min(test_file_data.len())];
    user.lock()
        .await
        .oc()
        .upload_chunk(UploadChunkRequest {
            upload_id: upload_id.clone(),
            chunk_data: Bytes::copy_from_slice(chunk),
            chunk_id: 0,
        })
        .await
        .unwrap();

    // Cancel the upload
    user.lock()
        .await
        .cancel_upload(upload_id.clone())
        .await
        .unwrap();

    // Trying to complete should fail
    assert!(
        user.lock()
            .await
            .oc()
            .complete_upload(CompleteUploadRequest {
                upload_id: upload_id.clone(),
            })
            .await
            .is_err()
    );

    // Verify no temp files remain after cancellation
    assert!(no_temp_files_remain(&app).await);

    app.async_drop().await;
}
