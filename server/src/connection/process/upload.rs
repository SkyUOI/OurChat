use crate::{
    client::requests::upload::Upload,
    component::EmailSender,
    connection::{Connection, response::UploadResponse},
    consts::{Bt, ID},
    db,
    utils::generate_random_string,
};
use anyhow::bail;
use sea_orm::DatabaseConnection;
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

impl<T: EmailSender> Connection<T> {
    pub async fn upload(
        id: ID,
        net_sender: &mpsc::Sender<Message>,
        json: &mut Upload,
        db_conn: &DatabaseConnection,
    ) -> anyhow::Result<(impl Future<Output = anyhow::Result<()>>, String)> {
        let ret = db::process::up_load(id, Bt(json.size), db_conn).await?;
        match ret {
            crate::client::requests::Status::Success => {
                let key = generate_key_name(&json.hash);
                let resp = UploadResponse::success(key.clone(), json.hash.clone());
                let send = async move {
                    net_sender.send(resp.into()).await?;
                    Ok(())
                };
                Ok((send, key))
            }
            _ => {
                bail!("unexpected error");
            }
        }
    }
}
