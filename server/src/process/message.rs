use pb::ourchat::msg_delivery::v1::Msg;
use prost::Message;

pub mod fetch_user_msg;
pub mod recall;
pub mod send_msg;

async fn transmit_msg(msg: Msg) -> anyhow::Result<()> {
    let mut buf = bytes::BytesMut::new();
    msg.encode(&mut buf)?;
    Ok(())
}
