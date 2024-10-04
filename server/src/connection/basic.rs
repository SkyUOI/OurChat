use super::NetSender;
use crate::client::response;

pub async fn send_error_msg(sender: impl NetSender, msg: impl Into<String>) -> anyhow::Result<()> {
    let error_resp = response::error_msg::ErrorMsgResponse::new(msg.into());
    sender.send(error_resp.into()).await?;
    Ok(())
}
