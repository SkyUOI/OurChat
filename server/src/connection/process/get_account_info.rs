use crate::{client::requests, connection::NetSender};

pub async fn get_account_info(
    net_sender: impl NetSender,
    request_data: requests::GetAccountInfoRequest,
) -> anyhow::Result<()> {
    Ok(())
}
