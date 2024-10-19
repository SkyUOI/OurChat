use super::NetSender;
use crate::client::{MsgConvert, response};

pub async fn send_error_msg(sender: impl NetSender, msg: impl Into<String>) -> anyhow::Result<()> {
    let error_resp = response::error_msg::ErrorMsgResponse::new(msg.into());
    sender.send(error_resp.to_msg()).await?;
    Ok(())
}
