use claims::assert_ok;
use client::ClientErr;

#[tokio::test]
async fn test_auth() {
    // ocid test
    let mut app = client::TestApp::new_with_launching_instance(None)
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();
    assert_ok!(user.lock().await.ocid_auth().await);

    let user = app.new_user().await.unwrap();
    // try wrong password
    claims::assert_err!(
        user.lock()
            .await
            .email_auth_internal("wrong password")
            .await
    );
    // email test
    assert_ok!(user.lock().await.email_auth().await);

    // try not found user
    let user = app.new_user().await.unwrap();
    user.lock().await.email = "wrong email".to_string();
    let e = user.lock().await.email_auth().await;
    if let Err(ClientErr::RpcStatus(e)) = e {
        assert_eq!(e.code(), tonic::Code::NotFound, "{:?}", e);
    } else {
        panic!("{:?}", e);
    }
    app.async_drop().await;
}

#[tokio::test]
async fn test_register() {
    // register two same users
    let mut app = client::TestApp::new_with_launching_instance(None)
        .await
        .unwrap();
    let user = app.new_user().await.unwrap();
    let e = user.lock().await.register().await;
    if let Err(ClientErr::RpcStatus(e)) = e {
        assert_eq!(e.code(), tonic::Code::AlreadyExists);
    } else {
        panic!("{:?}", e);
    }
    app.async_drop().await;
}
