use super::Connection;
use crate::{client::response, component::EmailSender};
use tokio::pin;
use tokio_tungstenite::tungstenite::protocol::Message;

impl<R: EmailSender> Connection<R> {
    pub async fn send_error_msg<T>(
        sender: impl Fn(Message) -> T,
        msg: impl Into<String>,
    ) -> anyhow::Result<()>
    where
        T: Future<Output = anyhow::Result<()>>,
    {
        let error_resp = response::error_msg::ErrorMsgResponse::new(msg.into());
        let future = sender(error_resp.into());
        pin!(future);
        (&mut future).await?;
        Ok(())
    }
}
