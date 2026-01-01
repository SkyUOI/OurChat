use client::TestApp;
use pb::service::ourchat::set_account_info::v1::SetSelfInfoRequest;

#[tokio::test]
async fn test_avatar() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let user_id = user.lock().await.id;
    let avatar = include_bytes!("../../test_data/test_avatar.png");
    let key = user.lock().await.post_file(avatar, None).await.unwrap();
    user.lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            avatar_key: Some(key.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    let res = app
        .ourchat_api_get(format!("avatar?user_id={}", user_id))
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_eq!(res.bytes().await.unwrap().as_ref(), avatar);
    app.async_drop().await;
}
