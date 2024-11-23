use tokio::fs::read_to_string;

#[tokio::test]
async fn test_upload() {
    let mut app = client::TestApp::new(None).await.unwrap();
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
    app.async_drop().await;
}
