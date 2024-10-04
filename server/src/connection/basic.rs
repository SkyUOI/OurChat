use super::{Connection, NetSender};
use crate::{client::response, component::EmailSender};

impl<R: EmailSender> Connection<R> {
    pub async fn send_error_msg(
        sender: impl NetSender,
        msg: impl Into<String>,
    ) -> anyhow::Result<()> {
        let error_resp = response::error_msg::ErrorMsgResponse::new(msg.into());
        sender.send(error_resp.into()).await?;
        Ok(())
    }
}
