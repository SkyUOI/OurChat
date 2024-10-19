use derive::db_compatibility;

use crate::{
    DbPool,
    client::requests,
    connection::{NetSender, UserInfo},
    consts::ID,
};

enum Privilege {
    Stranger,
    Owner,
}

pub async fn get_account_info(
    id: Option<&UserInfo>,
    net_sender: impl NetSender,
    request_data: requests::GetAccountInfoRequest,
    dbpool: &DbPool,
) -> anyhow::Result<()> {
    let privilege = match id {
        Some(id) => {
            if request_data.ocid == id.ocid {
                Privilege::Owner
            } else {
                Privilege::Stranger
            }
        }
        None => Privilege::Stranger,
    };

    Ok(())
}

#[db_compatibility]
async fn get_account_info_db() {}
