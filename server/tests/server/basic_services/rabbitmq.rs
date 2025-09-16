use std::time::Duration;

use client::{TestApp, oc_helper::user::FetchMsgErr};
use deadpool_lapin::lapin::options::ExchangeDeleteOptions;
use server::rabbitmq::{USER_MSG_BROADCAST_EXCHANGE, USER_MSG_DIRECT_EXCHANGE};

#[tokio::test]
async fn test_exchange_rebuild() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let connection = app.rabbitmq_pool.get().await.unwrap();
    let channel = connection.create_channel().await.unwrap();
    channel
        .exchange_delete(USER_MSG_DIRECT_EXCHANGE, ExchangeDeleteOptions::default())
        .await
        .unwrap();
    channel
        .exchange_delete(
            USER_MSG_BROADCAST_EXCHANGE,
            ExchangeDeleteOptions::default(),
        )
        .await
        .unwrap();
    match user
        .lock()
        .await
        .fetch_msgs()
        .set_timeout(Duration::from_secs(5))
        .fetch(0)
        .await
    {
        Err(FetchMsgErr::Timeout) => {}
        Err(e) => {
            panic!("{e}")
        }
        Ok(data) => {
            panic!("should fail but receive: {:?}", data)
        }
    }
    app.async_drop().await;
}
