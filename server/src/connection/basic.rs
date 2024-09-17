use super::{client_response, Connection};
use tokio_tungstenite::tungstenite::protocol::Message;

impl Connection {
    pub async fn send_error_msg(
        sender: impl async Fn(Message) -> anyhow::Result<()>,
        msg: impl Into<String>,
    ) -> anyhow::Result<()> {
        let error_resp = client_response::error_msg::ErrorMsgResponse::new(msg.into());
        sender(Message::Text(serde_json::to_string(&error_resp)?)).await?;
        Ok(())
    }
}
