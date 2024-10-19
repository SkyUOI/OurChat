use crate::{
    client::{
        MsgConvert,
        requests::{self, upload::Upload},
    },
    connection::response::UploadResponse,
    consts::{Bt, ID},
    shared_state,
    utils::generate_random_string,
};
use anyhow::bail;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

const PREFIX_LEN: usize = 20;

/// Generate a unique key name which refers to the file
/// # Details
/// Generate a 20-character random string, and then add the file's sha256 hash value
fn generate_key_name(hash: &str) -> String {
    let prefix: String = generate_random_string(PREFIX_LEN);
    format!("{}{}", prefix, hash)
}

#[derive::db_compatibility]
pub async fn add_file_record(
    id: ID,
    sz: Bt,
    db_connection: &DatabaseConnection,
) -> anyhow::Result<requests::Status> {
    use entities::user;
    let user_info = match user::Entity::find_by_id(id).one(db_connection).await? {
        Some(user) => user,
        None => {
            return Ok(requests::Status::ServerError);
        }
    };
    // first check if the limit has been reached
    let limit = shared_state::get_user_files_store_limit();
    let bytes_num: Bt = limit.into();
    let res_used: u64 = user_info.resource_used.try_into()?;
    let will_used = Bt(res_used + *sz);
    if will_used >= bytes_num {
        // reach the limit,delete some files to preserve the limit
        // TODO:clean files
    }
    let updated_res_lim = user_info.resource_used + 1;
    let mut user_info: user::ActiveModel = user_info.into();
    user_info.resource_used = ActiveValue::Set(updated_res_lim);
    user_info.update(db_connection).await?;
    Ok(requests::Status::Success)
}

pub async fn upload(
    id: ID,
    net_sender: &mpsc::Sender<Message>,
    json: &mut Upload,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<(impl Future<Output = anyhow::Result<()>>, String)> {
    let ret = add_file_record(id, Bt(json.size), db_conn).await?;
    match ret {
        crate::client::requests::Status::Success => {
            let key = generate_key_name(&json.hash);
            let resp = UploadResponse::success(key.clone(), json.hash.clone());
            let send = async move {
                net_sender.send(resp.to_msg()).await?;
                Ok(())
            };
            Ok((send, key))
        }
        _ => {
            bail!("unexpected error");
        }
    }
}
