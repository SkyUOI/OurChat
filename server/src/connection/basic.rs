use super::{Connection, client_response};
use tokio::pin;
use tokio_tungstenite::tungstenite::protocol::Message;

impl Connection {
    pub async fn send_error_msg<T>(
        sender: impl Fn(Message) -> T,
        msg: impl Into<String>,
    ) -> anyhow::Result<()>
    where
        T: Future<Output = anyhow::Result<()>>,
    {
        let error_resp = client_response::error_msg::ErrorMsgResponse::new(msg.into());
        let future = sender(Message::Text(serde_json::to_string(&error_resp)?));
        pin!(future);
        (&mut future).await?;
        Ok(())
    }
}
