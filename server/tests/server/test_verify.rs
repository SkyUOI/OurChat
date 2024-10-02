use server::requests;
use tokio_tungstenite::tungstenite::Message;

use crate::helper;

#[tokio::test]
async fn test_verify() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let req = requests::Verify::new("email".to_string());
    app.send(Message::Text(serde_json::to_string(&req).unwrap()))
        .await
        .unwrap();

    app.async_drop().await;
}
