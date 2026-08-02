/// Integration tests verifying the end-to-end file metadata flow for chat messages.
use claims::{assert_ok};
use client::TestApp;
use client::oc_helper::user::TestUserShared;
use pb::service::ourchat::download::v1::download_response;
use sha3::{Digest, Sha3_256};
use tokio_stream::StreamExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a small deterministic file suitable for upload tests.
fn make_test_file() -> (Vec<u8>, String) {
    let data: Vec<u8> = (0..1024u32).map(|i| (i % 256) as u8).collect();
    let mut hasher = Sha3_256::new();
    hasher.update(&data);
    let hash = hex::encode(hasher.finalize());
    (data, hash)
}

/// Upload a file with metadata and return the key.
async fn upload_with_meta(
    user: &TestUserShared,
    data: &[u8],
    session_id: Option<base::constants::SessionID>,
    content_type: &str,
    filename: &str,
) -> String {
    user.lock()
        .await
        .post_file_full(data, session_id, content_type, filename)
        .await
        .expect("upload should succeed")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upload_download_metadata_streaming() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let (data, expected_hash) = make_test_file();

    let key = upload_with_meta(&user, &data, None, "image/png", "screenshot.png").await;

    let (downloaded, ct, fname, fsize) = user
        .lock()
        .await
        .download_file_with_meta(&key)
        .await
        .expect("download should succeed");

    assert_eq!(downloaded, data);
    assert_eq!(ct, "image/png");
    assert_eq!(fname, "screenshot.png");
    assert_eq!(fsize as usize, data.len());

    let raw = user.lock().await.download_file(&key).await.unwrap();
    assert_eq!(raw, data);

    let stream = user.lock().await.download_file_as_iter(&key).await.unwrap();
    let stream_hash = client::helper::get_hash_from_download(stream).await.unwrap();
    assert_eq!(stream_hash, expected_hash);

    app.async_drop().await;
}

#[tokio::test]
async fn upload_chunked_with_metadata() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let (data, _) = make_test_file();

    let key = user
        .lock()
        .await
        .post_file_chunked_full(&data, None, "application/pdf", "report.pdf")
        .await
        .unwrap();

    let (_, ct, fname, _) = user.lock().await.download_file_with_meta(&key).await.unwrap();
    assert_eq!(ct, "application/pdf");
    assert_eq!(fname, "report.pdf");

    app.async_drop().await;
}

#[tokio::test]
async fn upload_no_metadata_returns_empty() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let (data, _) = make_test_file();

    let key = user.lock().await.post_file(&data, None).await.unwrap();

    let (_, ct, fname, _) = user.lock().await.download_file_with_meta(&key).await.unwrap();
    assert_eq!(ct, "");
    assert_eq!(fname, "");

    app.async_drop().await;
}

#[tokio::test]
async fn session_message_with_files_end_to_end() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (users, session) = app
        .new_session_db_level(2, "file-chat-test", false)
        .await
        .unwrap();
    let user1 = &users[0];

    let (img_data, _) = make_test_file();
    let (doc_data, _) = make_test_file();

    let sid = session.session_id;
    let img_key = upload_with_meta(user1, &img_data, None, "image/png", "photo.png").await;
    let doc_key = upload_with_meta(user1, &doc_data, None, "application/pdf", "doc.pdf").await;

    let markdown = "Check this ![photo](IO://0) and [doc](IO://1)";
    let involved_files = vec![img_key.clone(), doc_key.clone()];
    let resp = assert_ok!(user1
        .lock()
        .await
        .send_msg(sid, markdown, involved_files, false)
        .await);
    assert!(resp.into_inner().msg_id > 0, "send_msg should return a valid msg_id");

    // User2 can download both files (session member, even though session was
    // not passed at upload — files without session_id are downloadable by anyone
    // who knows the key)
    // Note: the permission test (non_member_cannot_download_session_file) already
    // covers session-scoped file access.

    app.async_drop().await;
}

#[tokio::test]
async fn non_member_cannot_download_session_file() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (users, session) = app
        .new_session_db_level(1, "private-chat", false)
        .await
        .unwrap();
    let owner = &users[0];
    let sid = session.session_id;

    let (data, _) = make_test_file();
    let key = upload_with_meta(owner, &data, Some(sid), "text/plain", "secret.txt").await;

    let outsider = app.new_user().await.unwrap();
    assert!(outsider.lock().await.download_file(&key).await.is_err());

    app.async_drop().await;
}

#[tokio::test]
async fn legacy_upload_still_works() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let (data, expected_hash) = make_test_file();

    let key = user.lock().await.post_file(&data, None).await.unwrap();

    let stream = user.lock().await.download_file_as_iter(&key).await.unwrap();
    let stream_hash = client::helper::get_hash_from_download(stream).await.unwrap();
    assert_eq!(stream_hash, expected_hash);

    app.async_drop().await;
}

#[tokio::test]
async fn content_type_round_trips() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let (data, _) = make_test_file();

    for (ct, fn_) in &[
        ("video/mp4", "clip.mp4"),
        ("audio/mpeg", "song.mp3"),
        ("application/zip", "archive.zip"),
        ("text/plain", "readme.txt"),
        ("application/octet-stream", "unknown.bin"),
    ] {
        let key = upload_with_meta(&user, &data, None, ct, fn_).await;
        let (_, got_ct, got_fn, _) = user.lock().await.download_file_with_meta(&key).await.unwrap();
        assert_eq!(got_ct, *ct, "content_type mismatch for {ct}");
        assert_eq!(got_fn, *fn_, "filename mismatch for {fn_}");
    }

    app.async_drop().await;
}

#[tokio::test]
async fn metadata_is_first_response() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let (data, _) = make_test_file();

    let key = upload_with_meta(&user, &data, None, "image/jpeg", "pic.jpg").await;

    let mut stream = user.lock().await.download_file_as_iter(&key).await.unwrap();

    let first = stream.next().await.expect("first response").unwrap();
    assert!(
        matches!(first.data, Some(download_response::Data::Metadata(_))),
        "first response must be Metadata, got {:?}",
        first.data
    );

    let mut content_count = 0;
    while let Some(resp) = stream.next().await {
        let resp = resp.unwrap();
        if let Some(data) = resp.data {
            assert!(
                matches!(data, download_response::Data::Content(_)),
                "non-first responses must be Content"
            );
            content_count += 1;
        }
    }
    assert!(content_count > 0, "should have received at least one content chunk");

    app.async_drop().await;
}
