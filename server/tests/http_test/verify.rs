use base::email_client::MockEmailSender;
use client::TestApp;
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
    let html_email_body = Arc::new(Mutex::new(None));
    let mock_html_body = html_email_body.clone();
    mock_smtp
        .expect_send()
        .times(1)
        .returning(move |_to, _title, body, html_body| {
            *mock_body.lock() = body;
            dbg!(&html_body);
            *mock_html_body.lock() = html_body;
            anyhow::Ok(())
        });
    let (config, args) = TestApp::get_test_config().unwrap();
    let mut app = TestApp::new_with_launching_instance_custom_cfg((config, args), |app| {
        app.http_launcher.as_mut().unwrap().email_client = Some(Box::new(mock_smtp));
    })
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
    let res = app.verify("wrong token").await.unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);

    // check email in mock server

    let link1 = {
        let binding = email_body.lock();
        let links = get_link(binding.as_str());

        assert_eq!(links.len(), 1);
        links.into_iter().next().unwrap()
    };
    if let Some(ref text) = *html_email_body.lock() {
        let link2 = get_link(text);
        assert_eq!(link2.len(), 2);
        assert_eq!(link1, link2[1]);
    }
    // check the link
    app.http_client
        .get(link1.as_str())
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    // get status
    let mut ret = ret.into_inner();
    ret.next().await.unwrap().unwrap();

    app.async_drop().await;
}

fn get_link(text: &str) -> Vec<String> {
    let link_finder = linkify::LinkFinder::new();
    link_finder
        .links(text)
        .filter(|x| *x.kind() == linkify::LinkKind::Url)
        .map(|x| x.as_str().to_string())
        .collect()
}
