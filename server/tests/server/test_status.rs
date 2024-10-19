use crate::helper;
use server::client::{
    MsgConvert, requests::get_status::GetStatus, response::get_status::GetStatusResponse,
};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_status() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user = app.new_user_logined().await.unwrap();
    let req = GetStatus::new();
    user.lock().await.send(req.to_msg()).await.unwrap();
    let ret = user.lock().await.get().await.unwrap();
    assert_eq!(ret, GetStatusResponse::normal().to_msg());
    app.async_drop().await;
}
