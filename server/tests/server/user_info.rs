use std::time::Duration;

use claims::{assert_err, assert_gt, assert_lt, assert_ok};
use client::TestApp;
use pb::service::ourchat::{
    friends::set_friend_info::v1::SetFriendInfoRequest,
    get_account_info::v1::{GetAccountInfoRequest, RequestValues},
    set_account_info::v1::SetSelfInfoRequest,
};
use pb::time::from_google_timestamp;
use server::process::error_msg::invalid::{OCID_TOO_LONG, STATUS_TOO_LONG, USERNAME};
use tokio::time::sleep;

#[tokio::test]
async fn get_user_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let time_before_register = app.get_timestamp().await;
    let user = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let time_after_register = app.get_timestamp().await;
    // request before logged in
    // don't have privileges
    let user_ocid = user.lock().await.ocid.clone();
    let user_id = user.lock().await.id;
    let user_name = user.lock().await.name.clone();
    let user_email = user.lock().await.email.clone();

    assert_err!(
        user2
            .lock()
            .await
            .oc()
            .get_account_info(GetAccountInfoRequest {
                id: Some(*user_id),
                request_values: vec![
                    RequestValues::Ocid.into(),
                    RequestValues::Email.into(),
                    RequestValues::UserName.into(),
                    RequestValues::RegisterTime.into(),
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
            id: None,
            request_values: vec![
                RequestValues::Ocid.into(),
                RequestValues::Email.into(),
                RequestValues::UserName.into(),
                RequestValues::Friends.into(),
                RequestValues::RegisterTime.into(),
            ],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.ocid, Some(user_ocid.clone().0));
    assert_eq!(ret.user_name, Some(user_name.clone()));
    assert_eq!(ret.email, Some(user_email.clone()));
    assert_eq!(ret.friends, Vec::<u64>::default());
    assert_gt!(
        from_google_timestamp(&ret.register_time.unwrap()).unwrap(),
        time_before_register
    );
    assert_lt!(
        from_google_timestamp(&ret.register_time.unwrap()).unwrap(),
        time_after_register
    );
    // TODO:add display_name test
    app.async_drop().await;
}

#[tokio::test]
async fn set_user_info() {
    // TODO: test avatar(especially reduce the refcnt)
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let new_name = "test_set_user_info".to_string();
    user.lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some(new_name.clone()),
            ocid: Some("modified_ocid".to_string()),
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
            id: None,
            request_values: vec![RequestValues::UserName.into(), RequestValues::Ocid.into()],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.user_name, Some(new_name.clone()));
    assert_eq!(&ret.ocid.unwrap(), "modified_ocid");
    // Too long ocid

    let err = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            ocid: Some("a".repeat(100)),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert_eq!(err.message(), OCID_TOO_LONG);

    app.async_drop().await;
}

#[tokio::test]
async fn set_friend_info() -> anyhow::Result<()> {
    let mut app = TestApp::new_with_launching_instance().await?;
    let user1 = app.new_user().await?;
    let user2 = app.new_user().await?;
    let user2_id = user2.lock().await.id;
    let new_name = "xxx";

    let ret = user1
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            id: Some(user2_id.into()),
            request_values: vec![RequestValues::DisplayName.into()],
        })
        .await?
        .into_inner();
    assert_eq!(ret.display_name.unwrap(), "");
    user1
        .lock()
        .await
        .oc()
        .set_friend_info(SetFriendInfoRequest {
            id: *user2_id,
            display_name: Some(new_name.to_owned()),
        })
        .await?;
    let ret = user1
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            id: Some(user2_id.into()),
            request_values: vec![RequestValues::DisplayName.into()],
        })
        .await?
        .into_inner();
    assert_eq!(ret.display_name.unwrap(), new_name);
    app.async_drop().await;
    Ok(())
}

#[tokio::test]
async fn set_user_info_validation() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.user_defined_status_expire_time = Duration::from_secs(5);
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args))
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();

    // Test empty user
    let err = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some("".to_string()),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.message(), USERNAME);

    // Test very long user
    let err = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some("a".repeat(65)),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.message(), USERNAME);

    // Test very long status
    let err = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_defined_status: Some("a".repeat(1000)),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(err.message(), STATUS_TOO_LONG);

    // Test successful set process
    assert_ok!(
        user.lock()
            .await
            .oc()
            .set_self_info(SetSelfInfoRequest {
                user_name: Some("valid_name".to_string()),
                user_defined_status: Some("valid status".to_string()),
                ..Default::default()
            })
            .await
    );

    // Test that the status' expire time is set correctly
    sleep(Duration::from_secs(6)).await;
    let mut ret = user
        .lock()
        .await
        .oc()
        .get_account_info(GetAccountInfoRequest {
            id: None,
            request_values: vec![RequestValues::Status.into()],
        })
        .await
        .unwrap();
    assert!(ret.get_mut().status.is_none());

    app.async_drop().await;
}
