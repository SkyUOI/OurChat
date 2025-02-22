use crate::helper;
use claims::assert_gt;
use http_server::matrix::route::client::VersionResponse;
use http_server::matrix::route::wellknown::{ClientResponse, SupportResponse};
use std::time::Duration;

#[tokio::test]
async fn version() {
    let mut app = helper::new(None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let res = app.matrix_api_get("client/versions").await.unwrap();
    dbg!(&res);
    let version: VersionResponse = res.json().await.unwrap();
    assert_gt!(version.versions.len(), 0);
    app.async_drop().await;
}

#[tokio::test]
async fn wellknown_support() {
    let mut app = helper::new(None).await.unwrap();
    let res = app.http_get(".well-known/matrix/support").await.unwrap();
    // dbg!(&res);
    let support: SupportResponse = res.json().await.unwrap();
    assert_eq!(
        support.contacts[0].matrix_id.clone().unwrap().0,
        format!("@limuy:{}", app.app_config.main_cfg.domain())
    );
    app.async_drop().await;
}

#[tokio::test]
async fn wellknown_client() {
    let mut app = helper::new(None).await.unwrap();
    let res = app.http_get(".well-known/matrix/client").await.unwrap();
    // dbg!(&res);
    let client: ClientResponse = res.json().await.unwrap();

    // Verify the homeserver URL matches our domain
    assert_eq!(
        client.m_homeserver.base_url.to_string(),
        format!("http://{}/", app.app_config.main_cfg.domain())
    );

    // Verify the identity server URL if it's configured
    if let Some(identity_server) = client.m_identity_server {
        assert!(!identity_server.base_url.to_string().is_empty());
    }

    app.async_drop().await;
}
