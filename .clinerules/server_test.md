# Server Test

An easy example:

```rust
async fn test_text_sent() {
    let mut app = client::TestApp::new_with_launching_instance()
        .await
        .unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, _b, _cc) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let ret: tonic::Response<msg_delivery::v1::SendMsgResponse> = a
        .lock()
        .await
        .send_msg(
            session.session_id,
            vec![OneMsg {
                data: Some(one_msg::Data::Text("hello".to_owned())),
            }],
            false,
        )
        .await
        .unwrap();
    let _msg_id = ret.into_inner().msg_id;
    app.async_drop().await;
}
```

The client is a helper crate, which is used to test the server.
