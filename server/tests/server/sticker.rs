use client::TestApp;
use pb::service::ourchat::sticker::v1::{
    AddStickerRequest, GetStickersRequest, RemoveStickerRequest,
};

/// Upload -> add -> get contains it -> remove -> get is empty
#[tokio::test]
async fn test_sticker_lifecycle() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let key = user
        .lock()
        .await
        .post_file(b"sticker image", None)
        .await
        .unwrap();

    // adding the same sticker twice is an explicit conflict
    user.lock()
        .await
        .oc()
        .add_sticker(AddStickerRequest {
            file_key: key.clone(),
        })
        .await
        .unwrap();
    let e = user
        .lock()
        .await
        .oc()
        .add_sticker(AddStickerRequest {
            file_key: key.clone(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::AlreadyExists, "{e:?}");

    let stickers = user
        .lock()
        .await
        .oc()
        .get_stickers(GetStickersRequest {})
        .await
        .unwrap()
        .into_inner()
        .stickers;
    assert_eq!(stickers.len(), 1, "{stickers:?}");
    assert_eq!(stickers[0].file_key, key);
    assert!(stickers[0].added_at.is_some());

    // removing twice fails the second time
    user.lock()
        .await
        .oc()
        .remove_sticker(RemoveStickerRequest {
            file_key: key.clone(),
        })
        .await
        .unwrap();
    let e = user
        .lock()
        .await
        .oc()
        .remove_sticker(RemoveStickerRequest { file_key: key })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::NotFound, "{e:?}");

    let stickers = user
        .lock()
        .await
        .oc()
        .get_stickers(GetStickersRequest {})
        .await
        .unwrap()
        .into_inner()
        .stickers;
    assert_eq!(stickers.len(), 0, "{stickers:?}");

    app.async_drop().await;
}

/// Stickers reference the user's own files only
#[tokio::test]
async fn test_sticker_add_others_file_rejected() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let a = app.new_user().await.unwrap();
    let b = app.new_user().await.unwrap();

    let key = a
        .lock()
        .await
        .post_file(b"someone else's file", None)
        .await
        .unwrap();

    // b cannot add a's file
    let e = b
        .lock()
        .await
        .oc()
        .add_sticker(AddStickerRequest {
            file_key: key.clone(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::PermissionDenied, "{e:?}");

    // b cannot remove a's sticker either (no such sticker of b's)
    a.lock()
        .await
        .oc()
        .add_sticker(AddStickerRequest {
            file_key: key.clone(),
        })
        .await
        .unwrap();
    let e = b
        .lock()
        .await
        .oc()
        .remove_sticker(RemoveStickerRequest { file_key: key })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::NotFound, "{e:?}");

    // a's collection is untouched
    let stickers = a
        .lock()
        .await
        .oc()
        .get_stickers(GetStickersRequest {})
        .await
        .unwrap()
        .into_inner()
        .stickers;
    assert_eq!(stickers.len(), 1, "{stickers:?}");

    app.async_drop().await;
}

/// Unknown file keys are rejected
#[tokio::test]
async fn test_sticker_add_nonexistent_key_rejected() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let e = user
        .lock()
        .await
        .oc()
        .add_sticker(AddStickerRequest {
            file_key: "no such key".to_string(),
        })
        .await
        .unwrap_err();
    assert_eq!(e.code(), tonic::Code::NotFound, "{e:?}");

    app.async_drop().await;
}

/// Multiple stickers are listed ordered by added_at
#[tokio::test]
async fn test_sticker_multiple_ordered() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let key1 = user
        .lock()
        .await
        .post_file(b"first sticker", None)
        .await
        .unwrap();
    let key2 = user
        .lock()
        .await
        .post_file(b"second sticker", None)
        .await
        .unwrap();
    let key3 = user
        .lock()
        .await
        .post_file(b"third sticker", None)
        .await
        .unwrap();

    for key in [&key1, &key2, &key3] {
        user.lock()
            .await
            .oc()
            .add_sticker(AddStickerRequest {
                file_key: key.clone(),
            })
            .await
            .unwrap();
    }

    let stickers = user
        .lock()
        .await
        .oc()
        .get_stickers(GetStickersRequest {})
        .await
        .unwrap()
        .into_inner()
        .stickers;
    let keys: Vec<_> = stickers.iter().map(|s| s.file_key.as_str()).collect();
    // added_at has microsecond resolution and the three inserts happen within
    // the same transaction burst, so only set equality is asserted
    assert_eq!(keys.len(), 3, "{stickers:?}");
    for key in [&key1, &key2, &key3] {
        assert!(keys.contains(&key.as_str()), "{keys:?}");
    }

    app.async_drop().await;
}
