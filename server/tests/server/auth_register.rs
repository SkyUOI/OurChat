use claims::assert_ok;
use client::oc_helper::ClientErr;

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
