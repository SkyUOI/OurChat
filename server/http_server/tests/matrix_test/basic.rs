use crate::helper;
use claims::assert_gt;
use http_server::matrix::route::client::VersionResponse;

#[tokio::test]
async fn version() {
    let mut app = helper::new(None).await.unwrap();
    let res = app.matrix_api_get("client/versions").await.unwrap();
    dbg!(&res);
    let version: VersionResponse = res.json().await.unwrap();
    assert_gt!(version.versions.len(), 0);
    app.async_drop().await;
}
