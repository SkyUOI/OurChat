use claims::assert_ok;

#[tokio::test]
async fn test_email_login() {
    let mut app = client::TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    // try wrong password
    claims::assert_err!(
        user.lock()
            .await
            .email_login_internal("wrong password")
            .await
    );
    assert_ok!(user.lock().await.email_login().await);
    app.async_drop().await;
}

#[tokio::test]
async fn test_ocid_login() {
    let mut app = client::TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    assert_ok!(user.lock().await.ocid_login().await);
    app.async_drop().await;
}
