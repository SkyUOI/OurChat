use claims::assert_ok;
use client::oc_helper::ClientErr;
use server::process::error_msg::{
    NOT_STRONG_PASSWORD,
    invalid::{EMAIL_ADDRESS, USERNAME},
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
    let e = user.lock().await.email_auth().await.unwrap_err();
    if let ClientErr::RpcStatus(e) = e {
        assert_eq!(e.code(), tonic::Code::NotFound, "{:?}", e);
    } else {
        panic!("{:?}", e);
    }
    app.async_drop().await;
}

#[tokio::test]
async fn register_account() {
    // register two same users
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();
    let e = user.lock().await.register().await.unwrap_err();
    if let ClientErr::RpcStatus(e) = e {
        assert_eq!(e.code(), tonic::Code::AlreadyExists);
    } else {
        panic!("{:?}", e);
    }
    app.async_drop().await;
}

#[tokio::test]
async fn register_validation() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();

    // Test username
    let user = app.new_user().await.unwrap();
    user.lock().await.name = "".to_string(); // empty usename
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(&err, USERNAME);

    user.lock().await.name = "a".repeat(65); // long username
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(&err, USERNAME);

    // Test password strength
    user.lock().await.name = "test_user".to_string();
    user.lock().await.password = "123456".to_string(); // weak password
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(&err, NOT_STRONG_PASSWORD);

    // Test email format
    user.lock().await.password = "StrongP@ssw0rd".to_string();
    user.lock().await.email = "invalid_email".to_string();
    let err = user.lock().await.register().await.unwrap_err();
    assert_status_message(&err, EMAIL_ADDRESS);

    app.async_drop().await;
}

// Helper function to check error message
fn assert_status_message(err: &ClientErr, expected_msg: &str) {
    if let ClientErr::RpcStatus(status) = err {
        assert_eq!(status.message(), expected_msg);
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    } else {
        panic!("Expected RpcStatus error, got: {:?}", err);
    }
}
