use crate::{
    component::EmailSender,
    connection::{Connection, client_response::UploadResponse},
    consts::{Bt, ID},
    requests::upload::Upload,
    server,
    utils::generate_random_string,
};
use anyhow::bail;
use sea_orm::DatabaseConnection;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

const PREFIX_LEN: usize = 20;

/// 生成独一无二的url名字
/// # Details
/// 先生成20位的随机字符串，再加上图片sha256哈希值
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
        let ret = server::process::up_load(id, Bt(json.size), db_conn).await?;
        match ret {
            crate::requests::Status::Success => {
                let key = generate_key_name(&json.hash);
                let resp = UploadResponse::success(key.clone(), json.hash.clone());
                let send = async move {
                    net_sender
                        .send(Message::Text(serde_json::to_string(&resp).unwrap()))
                        .await?;
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
