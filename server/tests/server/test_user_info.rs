use claims::assert_err;
use client::TestApp;
use server::pb::ourchat::{
    get_account_info::v1::{self, GetAccountInfoRequest, RequestValues},
    set_account_info::v1::{SetFriendInfoRequest, SetSelfInfoRequest},
};

#[tokio::test]
async fn test_get_user_info() {
    let mut app = TestApp::new_with_launching_instance(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    // request before logged in
    // don't have privileges
    let user_ocid = user.lock().await.ocid.clone();
    let user_name = user.lock().await.name.clone();
    let user_email = user.lock().await.email.clone();

    assert_err!(
        user2
            .lock()
            .await
            .oc()
            .get_account_info(GetAccountInfoRequest {
                ocid: user_ocid.clone(),
                request_values: vec![
                    RequestValues::Ocid.into(),
                    RequestValues::Email.into(),
                    RequestValues::UserName.into(),
                ],
            })
            .await
    );
    // now have privileges
    user.lock().await.ocid_auth().await.unwrap();
    let ret = user
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            ocid: user_ocid.clone(),
            request_values: vec![
                RequestValues::Ocid.into(),
                RequestValues::Email.into(),
                RequestValues::UserName.into(),
                RequestValues::Friends.into(),
            ],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.ocid, Some(user_ocid.clone()));
    assert_eq!(ret.user_name, Some(user_name.clone()));
    assert_eq!(ret.email, Some(user_email.clone()));
    assert_eq!(ret.friends, Vec::<String>::default());
    // TODO:add display_name test
    app.async_drop().await;
}

#[tokio::test]
async fn test_set_user_info() {
    let mut app = TestApp::new_with_launching_instance(None).await.unwrap();
    let user = app.new_user().await.unwrap();

    let ocid = user.lock().await.ocid.clone();

    let new_name = "test_set_user_info".to_string();
    let ret = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some(new_name.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    // get name
    let ret = user
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            ocid: ocid.clone(),
            request_values: vec![RequestValues::UserName.into()],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.user_name, Some(new_name.clone()));
    app.async_drop().await;
}

#[tokio::test]
async fn test_set_friend_info() {
    let mut app = TestApp::new_with_launching_instance(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user_ocid = user.lock().await.ocid.clone();
    let user2_ocid = user2.lock().await.ocid.clone();
    let new_name = "xxx";

    // now have privileges,but is no friends now
    let ret = user
        .lock()
        .await
        .oc()
        .set_friend_info(SetFriendInfoRequest {
            ocid: user2_ocid.clone(),
            display_name: Some(new_name.to_owned()),
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    // add friend
    app.async_drop().await;
}
