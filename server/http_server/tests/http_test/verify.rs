use base::email_client::MockEmailSender;
use client::TestApp;
use client::http_helper::TestHttpApp;
use parking_lot::Mutex;
use pb::service::auth::email_verify::v1::VerifyRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_verify() {
    // TODO:test reject in rabbitmq
    let mut mock_smtp = MockEmailSender::new();
    let email_body = Arc::new(Mutex::new(String::new()));
    let mock_body = email_body.clone();
    mock_smtp
        .expect_send()
        .times(1)
        .returning(move |_to, _title, body| {
            *mock_body.lock() = body;
            anyhow::Ok(())
        });
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let http_app = TestHttpApp::get_config().await.unwrap();
    let mut http_app = TestHttpApp::setup(
        http_app,
        Some(app.rmq_vhost.clone()),
        Some(Box::new(mock_smtp)),
    )
    .await
    .unwrap();
    let user = app.new_user().await.unwrap();
    let email = user.lock().await.email.clone();
    // Start Verify
    let ret = user
        .lock()
        .await
        .clients
        .auth
        .verify(VerifyRequest {
            email: email.clone(),
        })
        .await
        .unwrap();
    // check email
    for _ in 0..10 {
        let body = email_body.lock().is_empty();
        if body {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    // check the wrong link
    let res = http_app.verify("wrong token").await.unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);

    // check email in mock server
    let link_finder = linkify::LinkFinder::new();
    let link = {
        let email_body = email_body.lock();
        let links: Vec<_> = link_finder
            .links(&email_body)
            .filter(|x| *x.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links.first().unwrap().as_str().to_owned()
    };

    // check the link
    http_app
        .client
        .get(link)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    // get status
    let mut ret = ret.into_inner();
    ret.next().await.unwrap().unwrap();
    http_app.async_drop().await;
    app.async_drop().await;
}
