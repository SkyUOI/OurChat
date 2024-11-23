#[tokio::test]
async fn test_status_ws() {
    let mut app = client::TestApp::new(None).await.unwrap();
    let req = app.clients.basic.get_server_info(()).await.unwrap();
    let req = req.into_inner();
    assert_eq!(0, req.status);
    app.async_drop().await;
}
