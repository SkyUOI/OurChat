use crate::helper;
use claims::{assert_err, assert_ok};
use core::panic;
use parking_lot::Mutex;
use server::{
    component::MockEmailSender, connection::client_response, consts::MessageType, requests,
};
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_verify() {
    let mut mock_smtp = MockEmailSender::new();
    let email_body = Arc::new(Mutex::new(String::new()));
    let mock_body = email_body.clone();
    mock_smtp
        .expect_send::<&str>()
        .times(1)
        .returning(move |_to, _title, body| {
            *mock_body.lock() = body;
            anyhow::Ok(())
        });
    let mut app = helper::TestApp::new(Some(mock_smtp)).await.unwrap();
    claims::assert_some!(app.app_shared.email_client.as_ref());
    let req = requests::Verify::new(app.user.email.clone());
    app.send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();
    // Send successfully
    let ret: client_response::VerifyResponse =
        serde_json::from_str(&app.get().await.unwrap().to_string()).unwrap();
    // check email
    for _ in 0..10 {
        let body = email_body.lock().is_empty();
        if body {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert_eq!(ret.code, MessageType::VerifyRes);
    assert_eq!(ret.status, requests::Status::Success);
    // check wrong link
    let res = app.verify("wrong token").await.unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_err!(app.email_login().await);

    // check email in mock server
    let link_finder = linkify::LinkFinder::new();
    let email_body = email_body.lock();
    let links: Vec<_> = link_finder
        .links(&email_body)
        .filter(|x| *x.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(), 1);
    let link = links.first().unwrap().as_str().to_owned();
    drop(email_body);

    // check link
    app.http_client
        .get(link)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_ok!(app.email_login().await);
    app.async_drop().await;
}
