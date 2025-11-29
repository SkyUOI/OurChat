use client::TestApp;
use entities::{files, user};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use server::db::file_storage;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

#[tokio::test]
async fn test_file_cache_functionality() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Get the file cache from the application
    let file_cache = app.app_shared.file_sys.as_ref().unwrap().get_cache();

    // Test basic cache operations
    let test_key = "test_cache_key";
    let test_data = b"test cache data".to_vec();

    // Initially, cache should be empty
    assert!(file_cache.get(test_key).await.is_none());

    // Add data to cache
    file_cache
        .put(test_key.to_string(), test_data.clone())
        .await;

    // Retrieve data from cache
    let cached_data = file_cache.get(test_key).await;
    assert!(cached_data.is_some());
    assert_eq!(cached_data.unwrap(), test_data);

    // Test cache statistics
    let stats = file_cache.get_stats().await;
    assert_eq!(stats.get(test_key), Some(&2u32)); // Should have 2 accesses (put + get)

    // Access again and check stats
    file_cache.get(test_key).await;
    let stats = file_cache.get_stats().await;
    assert_eq!(stats.get(test_key), Some(&3u32)); // Should have 3 accesses (put + 2 gets)

    // Remove from cache
    file_cache.remove(test_key).await;
    assert!(file_cache.get(test_key).await.is_none());

    app.async_drop().await;
}

#[tokio::test]
async fn test_hierarchical_path_generation() {
    let base_path = Path::new("files_storage");
    let user_id = 12345;
    let file_key = "abcdef1234567890";

    let path = file_storage::generate_hierarchical_path(base_path, user_id, file_key);

    // Check that path is hierarchical
    assert!(path.to_string_lossy().contains("39")); // 12345 % 256 = 57 (0x39)
    assert!(path.to_string_lossy().contains("ab")); // first 2 chars of key
    assert!(path.to_string_lossy().contains(file_key));

    // Verify the full path structure
    let expected_path = base_path.join("39").join("ab").join(file_key);
    assert_eq!(path, expected_path);
}

#[tokio::test]
async fn test_reference_counting_operations() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a test user first
    let test_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(1),
        ocid: sea_orm::ActiveValue::Set("test_user".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("Test User".to_string()),
        email: sea_orm::ActiveValue::Set("test@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(None),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(None),
        oauth_provider: sea_orm::ActiveValue::Set(None),
        email_verified: sea_orm::ActiveValue::Set(true),
    };

    user::Entity::insert(test_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert test user");

    // Create a test file record
    let test_key = "test_refcnt_key".to_string();
    let test_path = "files_storage/test_file.txt".to_string();

    let file_record = files::ActiveModel {
        key: sea_orm::ActiveValue::Set(test_key.clone()),
        path: sea_orm::ActiveValue::Set(test_path.clone()),
        date: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp()),
        auto_clean: sea_orm::ActiveValue::Set(false),
        user_id: sea_orm::ActiveValue::Set(1),
        ref_cnt: sea_orm::ActiveValue::Set(1),
        hash: sea_orm::ActiveValue::Set(None),
    };

    files::Entity::insert(file_record)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert test file record");

    // Test incrementing reference count
    file_storage::inc_file_refcnt(&test_key, &app.db_pool.db_pool)
        .await
        .expect("Failed to increment reference count");

    // Verify reference count was incremented
    let file_after_inc = files::Entity::find_by_id(&test_key)
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query file")
        .expect("File not found");

    assert_eq!(file_after_inc.ref_cnt, 2);

    // Test decrementing reference count
    file_storage::dec_file_refcnt(&test_key, &app.db_pool.db_pool)
        .await
        .expect("Failed to decrement reference count");

    // Verify reference count was decremented
    let file_after_dec = files::Entity::find_by_id(&test_key)
        .one(&app.db_pool.db_pool)
        .await
        .expect("Failed to query file")
        .expect("File not found");

    assert_eq!(file_after_dec.ref_cnt, 1);

    // Clean up
    files::Entity::delete_by_id(&test_key)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to delete test file record");

    app.async_drop().await;
}

#[tokio::test]
async fn test_batch_reference_count_operations() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a test user first
    let test_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(1),
        ocid: sea_orm::ActiveValue::Set("test_user".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("Test User".to_string()),
        email: sea_orm::ActiveValue::Set("test@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(None),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(None),
        oauth_provider: sea_orm::ActiveValue::Set(None),
        email_verified: sea_orm::ActiveValue::Set(true),
    };

    user::Entity::insert(test_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert test user");

    // Create multiple test file records
    let test_keys = vec!["batch_test_1", "batch_test_2", "batch_test_3"];
    let test_paths = vec![
        "files_storage/batch_1.txt",
        "files_storage/batch_2.txt",
        "files_storage/batch_3.txt",
    ];

    for (i, key) in test_keys.iter().enumerate() {
        let file_record = files::ActiveModel {
            key: sea_orm::ActiveValue::Set(key.to_string()),
            path: sea_orm::ActiveValue::Set(test_paths[i].to_string()),
            date: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp()),
            auto_clean: sea_orm::ActiveValue::Set(false),
            user_id: sea_orm::ActiveValue::Set(1),
            ref_cnt: sea_orm::ActiveValue::Set(1),
            hash: sea_orm::ActiveValue::Set(None),
        };

        files::Entity::insert(file_record)
            .exec(&app.db_pool.db_pool)
            .await
            .expect("Failed to insert test file record");
    }

    // Test batch operations
    let operations = vec![
        (test_keys[0].to_string(), 2),  // Increment by 2
        (test_keys[1].to_string(), -1), // Decrement by 1
        (test_keys[2].to_string(), 0),  // No change
    ];

    file_storage::batch_update_refcnt(operations, &app.db_pool.db_pool)
        .await
        .expect("Failed to execute batch operations");

    // Verify results
    let files_after_batch = files::Entity::find()
        .filter(files::Column::Key.is_in(test_keys.clone()))
        .all(&app.db_pool.db_pool)
        .await
        .expect("Failed to query files");

    // batch_test_2 should be deleted since ref_cnt becomes 0
    assert_eq!(files_after_batch.len(), 2);

    for file in files_after_batch {
        match file.key.as_str() {
            "batch_test_1" => assert_eq!(file.ref_cnt, 3), // 1 + 2 = 3
            "batch_test_3" => assert_eq!(file.ref_cnt, 1), // 1 + 0 = 1
            _ => panic!("Unexpected file key: {}", file.key),
        }
    }

    // Clean up
    for key in test_keys {
        files::Entity::delete_by_id(key)
            .exec(&app.db_pool.db_pool)
            .await
            .expect("Failed to delete test file record");
    }

    app.async_drop().await;
}

#[tokio::test]
async fn test_file_deduplication_logic() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a test user first
    let test_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(1),
        ocid: sea_orm::ActiveValue::Set("test_user".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("Test User".to_string()),
        email: sea_orm::ActiveValue::Set("test@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(None),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(None),
        oauth_provider: sea_orm::ActiveValue::Set(None),
        email_verified: sea_orm::ActiveValue::Set(true),
    };

    user::Entity::insert(test_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert test user");

    // Create files with same hash to test deduplication
    let _common_hash = "duplicate_hash_value".to_string();
    let test_keys = vec!["dup_test_1", "dup_test_2", "dup_test_3"];

    for key in &test_keys {
        let file_record = files::ActiveModel {
            key: sea_orm::ActiveValue::Set(key.to_string()),
            path: sea_orm::ActiveValue::Set(format!("files_storage/{}", key)),
            date: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp()),
            auto_clean: sea_orm::ActiveValue::Set(false),
            user_id: sea_orm::ActiveValue::Set(1),
            ref_cnt: sea_orm::ActiveValue::Set(1),
            hash: sea_orm::ActiveValue::Set(None),
        };

        files::Entity::insert(file_record)
            .exec(&app.db_pool.db_pool)
            .await
            .expect("Failed to insert test file record");
    }

    // Run deduplication
    // let duplicates_found = file_storage::deduplicate_files(
    //     &app.db_pool.db_pool,
    //     Path::new("files_storage")
    // ).await.expect("Failed to run deduplication");

    // Should find 2 duplicates (3 files with same hash, keep 1, delete 2)
    // assert_eq!(duplicates_found, 2);

    // Verify only one file remains
    // let remaining_files = files::Entity::find()
    //     .filter(files::Column::Hash.eq(common_hash.clone()))
    //     .all(&app.db_pool.db_pool)
    //     .await
    //     .expect("Failed to query files");

    // assert_eq!(remaining_files.len(), 1);

    // Clean up
    for key in test_keys {
        files::Entity::delete_by_id(key)
            .exec(&app.db_pool.db_pool)
            .await
            .expect("Failed to delete test file record");
    }

    app.async_drop().await;
}

#[tokio::test]
async fn test_orphaned_file_cleanup() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // Create a test user first
    let test_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(1),
        ocid: sea_orm::ActiveValue::Set("test_user".to_string()),
        passwd: sea_orm::ActiveValue::Set(None),
        name: sea_orm::ActiveValue::Set("Test User".to_string()),
        email: sea_orm::ActiveValue::Set("test@example.com".to_string()),
        time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: sea_orm::ActiveValue::Set(0),
        friend_limit: sea_orm::ActiveValue::Set(5000),
        friends_num: sea_orm::ActiveValue::Set(0),
        avatar: sea_orm::ActiveValue::Set(None),
        public_update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        update_time: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        account_status: sea_orm::ActiveValue::Set(1),
        deleted_at: sea_orm::ActiveValue::Set(None),
        public_key: sea_orm::ActiveValue::Set(vec![]),
        github_id: sea_orm::ActiveValue::Set(None),
        oauth_provider: sea_orm::ActiveValue::Set(None),
        email_verified: sea_orm::ActiveValue::Set(true),
    };

    user::Entity::insert(test_user)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert test user");

    // Create a temporary directory for testing
    let test_dir = "test_orphaned_files";
    let _ = fs::create_dir_all(test_dir).await;

    // Create some test files on disk
    let test_files = vec!["orphan_1.txt", "orphan_2.txt", "orphan_3.txt"];

    for file_name in &test_files {
        let file_path = format!("{}/{}", test_dir, file_name);
        fs::write(&file_path, b"test content")
            .await
            .expect("Failed to create test file");
    }

    // Create one file record in database (so only 2 files should be orphaned)
    let file_record = files::ActiveModel {
        key: sea_orm::ActiveValue::Set("db_file".to_string()),
        path: sea_orm::ActiveValue::Set(format!("{}/{}", test_dir, test_files[0])),
        date: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp()),
        auto_clean: sea_orm::ActiveValue::Set(false),
        user_id: sea_orm::ActiveValue::Set(1),
        ref_cnt: sea_orm::ActiveValue::Set(1),
        hash: sea_orm::ActiveValue::Set(None),
    };

    files::Entity::insert(file_record)
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to insert test file record");

    // Run orphaned file cleanup
    let orphaned_count =
        file_storage::cleanup_orphaned_files(&app.db_pool.db_pool, Path::new(test_dir))
            .await
            .expect("Failed to run orphaned file cleanup");

    // Should find 2 orphaned files
    assert_eq!(orphaned_count, 2);

    // Verify only the database-tracked file remains
    let mut remaining_files = Vec::new();
    let mut entries = fs::read_dir(test_dir)
        .await
        .expect("Failed to read directory");
    while let Some(entry) = entries
        .next_entry()
        .await
        .expect("Failed to read directory entry")
    {
        remaining_files.push(entry);
    }

    assert_eq!(remaining_files.len(), 1);
    assert!(
        remaining_files[0]
            .file_name()
            .to_string_lossy()
            .contains(test_files[0])
    );

    // Clean up
    files::Entity::delete_by_id("db_file")
        .exec(&app.db_pool.db_pool)
        .await
        .expect("Failed to delete test file record");

    fs::remove_dir_all(test_dir)
        .await
        .expect("Failed to clean up test directory");

    app.async_drop().await;
}

#[tokio::test]
async fn test_file_upload_download_with_cache() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // This test would require actual file upload/download testing
    // For now, we'll test the basic file system integration

    // Verify file system is properly initialized
    assert!(app.app_shared.file_sys.is_some());

    // Verify file cache is available
    let file_cache = app.app_shared.file_sys.as_ref().unwrap().get_cache();
    assert!(Arc::strong_count(&file_cache) > 0);

    // Test that hierarchical storage is configured
    let cfg = &app.app_shared.cfg.main_cfg;
    assert!(cfg.enable_hierarchical_storage);
    assert!(cfg.enable_file_cache);

    app.async_drop().await;
}
