use crate::helper;
use server::client::{MsgConvert, requests::GetStatus, response::get_status::GetStatusResponse};

#[tokio::test]
async fn test_status_ws() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user = app.new_user_logined().await.unwrap();
    let req = GetStatus::new();
    user.lock().await.send(req.to_msg()).await.unwrap();
    let ret = user.lock().await.recv().await.unwrap();
    assert_eq!(ret, GetStatusResponse::normal().to_msg());
    app.async_drop().await;
}
