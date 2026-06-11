use std::time::Duration;

use claims::{assert_err, assert_gt, assert_lt, assert_ok};
use client::TestApp;
use pb::{
    service::ourchat::{
        friends::set_friend_info::v1::SetFriendInfoRequest,
        get_account_info::v1::{GetAccountInfoRequest, QueryValues},
        set_account_info::v1::SetSelfInfoRequest,
    },
    time::TimeStampUtc,
};
use sea_orm::TransactionTrait;
use server::process::{
    db,
    error_msg::invalid::{OCID_TOO_LONG, STATUS_TOO_LONG, USERNAME},
};
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
            .get_account_info(
                user_id,
                vec![
                    QueryValues::Ocid,
                    QueryValues::Email,
                    QueryValues::UserName,
                    QueryValues::RegisterTime,
                ],
            )
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
                QueryValues::Ocid.into(),
                QueryValues::Email.into(),
                QueryValues::UserName.into(),
                QueryValues::Friends.into(),
                QueryValues::RegisterTime.into(),
            ],
        })
        .await
        .unwrap();
    let ret = ret.into_inner();
    assert_eq!(ret.ocid, Some(user_ocid.clone().0));
    assert_eq!(ret.user_name, Some(user_name.clone()));
    assert_eq!(ret.email, Some(user_email.clone()));
    assert_eq!(ret.friends, Vec::<u64>::default());
    let tmp: TimeStampUtc = ret.register_time.unwrap().try_into().unwrap();
    assert_gt!(tmp, time_before_register);
    let tmp: TimeStampUtc = ret.register_time.unwrap().try_into().unwrap();
    assert_lt!(tmp, time_after_register);
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
            request_values: vec![QueryValues::UserName.into(), QueryValues::Ocid.into()],
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
        .get_account_info(user2_id, vec![QueryValues::DisplayName])
        .await?;
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
        .get_account_info(user2_id, vec![QueryValues::DisplayName])
        .await?;
    assert_eq!(ret.display_name.unwrap(), new_name);
    app.async_drop().await;
    Ok(())
}

#[tokio::test]
async fn set_user_info_validation() {
    let (mut config, args) = TestApp::get_test_config().unwrap();
    config.main_cfg.user_defined_status_expire_time = Duration::from_secs(5);
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
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
            request_values: vec![QueryValues::Status.into()],
        })
        .await
        .unwrap();
    assert!(ret.get_mut().status.is_none());

    app.async_drop().await;
}

#[tokio::test]
async fn different_user_get_info() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let (ocid1, ocid2) = (
        user1.lock().await.ocid.clone(),
        user2.lock().await.ocid.clone(),
    );
    let (user1_id, user2_id) = (user1.lock().await.id, user2.lock().await.id);
    assert_eq!(
        user2
            .lock()
            .await
            .get_self_info(vec![QueryValues::Ocid])
            .await
            .unwrap()
            .ocid
            .unwrap(),
        ocid2.0
    );
    assert_eq!(
        user1
            .lock()
            .await
            .get_self_info(vec![QueryValues::Ocid])
            .await
            .unwrap()
            .ocid
            .unwrap(),
        ocid1.0
    );
    assert_eq!(
        user1
            .lock()
            .await
            .get_account_info(user2_id, vec![QueryValues::Ocid])
            .await
            .unwrap()
            .ocid
            .unwrap(),
        ocid2.0
    );
    assert_eq!(
        user2
            .lock()
            .await
            .get_account_info(user1_id, vec![QueryValues::Ocid],)
            .await
            .unwrap()
            .ocid
            .unwrap(),
        ocid1.0
    );
    app.async_drop().await;
}

#[tokio::test]
async fn join_in_session_with_update_time_changed() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user1_id = user1.lock().await.id;
    let (_, session) = app.new_session_db_level(1, "test", false).await.unwrap();
    let get_timestamp = async || user1.lock().await.get_update_timestamp().await.unwrap();
    let origin_timestamp = get_timestamp().await;
    let transaction = app.get_db_connection().begin().await.unwrap();
    db::join_in_session(session.session_id, user1_id, None, &transaction)
        .await
        .unwrap();
    transaction.commit().await.unwrap();
    let now_timestamp = get_timestamp().await;
    assert_gt!(now_timestamp, origin_timestamp);
    app.async_drop().await;
}

// ── email_visible privacy tests ──

/// New users should have email_visible = false by default (migration default).
#[tokio::test]
async fn email_visible_default_false() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    let ret = user
        .lock()
        .await
        .get_self_info(vec![QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email_visible, Some(false));

    app.async_drop().await;
}

/// Owners can always see their own email regardless of email_visible value.
#[tokio::test]
async fn email_privacy_owner_always_sees_email() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let user_email = user.lock().await.email.clone();

    // email_visible is false by default, but owner should still see their email
    let ret = user
        .lock()
        .await
        .get_self_info(vec![QueryValues::Email, QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email.as_ref(), Some(&user_email));
    assert_eq!(ret.email_visible, Some(false));

    // Set email_visible to true, owner should still see email
    user.lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            email_visible: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();

    let ret = user
        .lock()
        .await
        .get_self_info(vec![QueryValues::Email, QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email.as_ref(), Some(&user_email));
    assert_eq!(ret.email_visible, Some(true));

    app.async_drop().await;
}

/// Strangers cannot see email when email_visible is false.
#[tokio::test]
async fn email_privacy_stranger_blocked() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user2_id = user2.lock().await.id;

    // user2 has email_visible = false by default, so user1 should get PermissionDenied
    let err = user1
        .lock()
        .await
        .get_account_info(user2_id, vec![QueryValues::Email])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("permission"));

    app.async_drop().await;
}

/// Strangers can see email when email_visible is true.
#[tokio::test]
async fn email_privacy_stranger_allowed() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user2_id = user2.lock().await.id;
    let user2_email = user2.lock().await.email.clone();

    // user2 enables email visibility
    user2
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            email_visible: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();

    // user1 should now see user2's email
    let ret = user1
        .lock()
        .await
        .get_account_info(user2_id, vec![QueryValues::Email])
        .await
        .unwrap();
    assert_eq!(ret.email, Some(user2_email));

    app.async_drop().await;
}

/// email_visible itself is always publicly queryable.
#[tokio::test]
async fn email_visible_always_public() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user1 = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user2_id = user2.lock().await.id;

    // Stranger can always see email_visible status
    let ret = user1
        .lock()
        .await
        .get_account_info(user2_id, vec![QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email_visible, Some(false));

    // user2 toggles it on
    user2
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            email_visible: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();

    // user1 should see the update
    let ret = user1
        .lock()
        .await
        .get_account_info(user2_id, vec![QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email_visible, Some(true));

    app.async_drop().await;
}

/// Toggling email_visible via SetSelfInfo should be reflected in get_account_info.
#[tokio::test]
async fn set_email_visible_toggle() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();

    // Initially false
    let ret = user
        .lock()
        .await
        .get_self_info(vec![QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email_visible, Some(false));

    // Toggle to true
    user.lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            email_visible: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    let ret = user
        .lock()
        .await
        .get_self_info(vec![QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email_visible, Some(true));

    // Toggle back to false
    user.lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            email_visible: Some(false),
            ..Default::default()
        })
        .await
        .unwrap();
    let ret = user
        .lock()
        .await
        .get_self_info(vec![QueryValues::EmailVisible])
        .await
        .unwrap();
    assert_eq!(ret.email_visible, Some(false));

    // Setting again to false (no-op) should not error
    assert_ok!(
        user.lock()
            .await
            .oc()
            .set_self_info(SetSelfInfoRequest {
                email_visible: Some(false),
                ..Default::default()
            })
            .await
    );

    app.async_drop().await;
}
