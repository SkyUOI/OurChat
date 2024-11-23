use crate::helper;
use claims::{assert_err, assert_ok};
use core::panic;
use parking_lot::Mutex;
use server::component::MockEmailSender;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_verify() {
    // let mut mock_smtp = MockEmailSender::new();
    // let email_body = Arc::new(Mutex::new(String::new()));
    // let mock_body = email_body.clone();
    // mock_smtp
    //     .expect_send::<String>()
    //     .times(1)
    //     .returning(move |_to, _title, body| {
    //         *mock_body.lock() = body;
    //         anyhow::Ok(())
    //     });
    // let mut app = helper::TestApp::new(Some(mock_smtp)).await.unwrap();
    // let user = app.new_user().await.unwrap();
    // claims::assert_some!(app.app_shared.email_client.as_ref());
    // let req = requests::VerifyRequest::new(user.lock().await.email.clone());
    // user.lock().await.send(req.to_msg()).await.unwrap();
    // // Send successfully
    // let ret: response::VerifyResponse =
    //     serde_json::from_str(&user.lock().await.recv().await.unwrap().to_string()).unwrap();
    // // check email
    // for _ in 0..10 {
    //     let body = email_body.lock().is_empty();
    //     if body {
    //         break;
    //     }
    //     tokio::time::sleep(Duration::from_millis(100)).await;
    // }

    // assert_eq!(ret.code, MessageType::VerifyRes);
    // assert_eq!(ret.status, requests::Status::Success);
    // // check wrong link
    // let res = app.verify("wrong token").await.unwrap();
    // assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);
    // assert_err!(user.lock().await.email_login().await);

    // // check email in mock server
    // let link_finder = linkify::LinkFinder::new();
    // let link = {
    //     let email_body = email_body.lock();
    //     let links: Vec<_> = link_finder
    //         .links(&email_body)
    //         .filter(|x| *x.kind() == linkify::LinkKind::Url)
    //         .collect();
    //     assert_eq!(links.len(), 1);
    //     links.first().unwrap().as_str().to_owned()
    // };

    // // check link
    // app.http_client
    //     .get(link)
    //     .send()
    //     .await
    //     .unwrap()
    //     .error_for_status()
    //     .unwrap();
    // // get status
    // let status = serde_json::from_str::<response::VerifyResponse>(
    //     &user.lock().await.recv().await.unwrap().to_string(),
    // )
    // .unwrap();
    // assert_eq!(status.code, MessageType::VerifyRes);
    // assert_eq!(status.status, requests::Status::Success);
    // assert_ok!(user.lock().await.email_login().await);
    // app.async_drop().await;
}
