use claims::assert_ok;
use client::oc_helper::ClientErr;
use pb::service::ourchat::set_account_info::v1::SetSelfInfoRequest;
use server::process::error_msg::{
    ACCOUNT_DELETED, NOT_STRONG_PASSWORD,
    invalid::{EMAIL_ADDRESS, USERNAME},
    not_found,
};

#[tokio::test]
async fn auth_token() {
    // ocid test
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();
    assert_ok!(user.lock().await.ocid_auth().await);

    let user = app.new_user().await.unwrap();
    // try the wrong password
    claims::assert_err!(
        user.lock()
            .await
            .email_auth_internal("wrong password")
            .await
    );
    // email test
    assert_ok!(user.lock().await.email_auth().await);

    // try a user which not exists
    let user = app.new_user().await.unwrap();
    user.lock().await.email = "wrong email".to_string();
    let e = user
        .lock()
        .await
        .email_auth()
        .await
        .unwrap_err()
        .unwrap_rpc_status();
    assert_eq!(e.code(), tonic::Code::NotFound, "{e:?}");
    app.async_drop().await;
}

#[tokio::test]
async fn register_account() {
    // register two same users
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();
    let e = user
        .lock()
        .await
        .register()
        .await
        .unwrap_err()
        .unwrap_rpc_status();
    assert_eq!(e.code(), tonic::Code::AlreadyExists);
    app.async_drop().await;
}

#[tokio::test]
async fn register_validation() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Test username
    let user = app.new_user().await.unwrap();
    user.lock().await.name = "".to_string(); // empty username
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(err, USERNAME);

    user.lock().await.name = "a".repeat(65); // long username
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(err, USERNAME);

    // Test password strength
    user.lock().await.name = "test_user".to_string();
    user.lock().await.password = "123456".to_string(); // weak password
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(err, NOT_STRONG_PASSWORD);

    // Test email format
    user.lock().await.password = "StrongP@ssw0rd".to_string();
    user.lock().await.email = "invalid_email".to_string();
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(err, EMAIL_ADDRESS);

    app.async_drop().await;
}

#[tokio::test]
async fn unregister_account_with_disable_policy() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Create and authenticate a user
    let user = app.new_user().await.unwrap();
    assert_ok!(user.lock().await.email_auth().await);

    // Test successful unregister
    assert_ok!(user.lock().await.unregister().await);

    // Try to unregister again
    let status = user.lock().await.unregister().await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);
    assert_eq!(status.message(), ACCOUNT_DELETED);

    // Try to set self-info, should be failed
    let status = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some("test_user".to_string()),
            ocid: Some("modified_ocid".to_string()),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);
    assert_eq!(status.message(), ACCOUNT_DELETED);

    app.async_drop().await;
}

#[tokio::test]
async fn unregister_account_with_delete_policy() {
    let (mut config, args) = client::TestApp::get_test_config().unwrap();
    config.main_cfg.unregister_policy = server::config::UnregisterPolicy::Delete;
    let mut app = client::TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Create and authenticate a user
    let user = app.new_user().await.unwrap();
    assert_ok!(user.lock().await.email_auth().await);

    // Test successful unregister
    assert_ok!(user.lock().await.unregister().await);

    // Try to unregister again
    let status = user.lock().await.unregister().await.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);
    assert_eq!(status.message(), not_found::USER);

    // Try to set self-info, should be failed
    let status = user
        .lock()
        .await
        .oc()
        .set_self_info(SetSelfInfoRequest {
            user_name: Some("test_user".to_string()),
            ocid: Some("modified_ocid".to_string()),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);
    assert_eq!(status.message(), not_found::USER);

    app.async_drop().await;
}

// Helper function to check the error message
fn assert_status_message(err: ClientErr, expected_msg: &str) {
    let err = err.unwrap_rpc_status();
    assert_eq!(err.message(), expected_msg);
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn register_with_default_session() {
    use base::consts::SessionID;
    use server::db::session::{get_session_by_id, in_session};

    let (mut config, args) = client::TestApp::get_test_config().unwrap();
    // Set default session ID
    config.main_cfg.default_session = Some(SessionID(12345));
    let mut app = client::TestApp::new_with_launching_instance_custom_cfg((config, args), |_| {})
        .await
        .unwrap();

    // Register a new user
    let user = app.new_user().await.unwrap();
    let user_id = user.lock().await.id;

    // Verify user is in the default session
    let default_session_id = SessionID(12345);
    assert!(
        in_session(user_id, default_session_id, app.get_db_connection())
            .await
            .unwrap()
    );

    // Verify session was created (if it didn't exist)
    let session = get_session_by_id(default_session_id, app.get_db_connection())
        .await
        .unwrap()
        .expect("default session should exist");
    assert_eq!(session.session_id, default_session_id.0 as i64);
    assert_eq!(session.size, 1); // only the new user
    assert!(!session.e2ee_on); // e2ee_on should be false for default session creation

    // Register another user and verify they also join the same session
    let user2 = app.new_user().await.unwrap();
    let user2_id = user2.lock().await.id;
    assert!(
        in_session(user2_id, default_session_id, app.get_db_connection())
            .await
            .unwrap()
    );

    // Verify session size increased
    let session = get_session_by_id(default_session_id, app.get_db_connection())
        .await
        .unwrap()
        .expect("default session should exist");
    assert_eq!(session.size, 2); // both users

    app.async_drop().await;
}
