use crate::{
    component::EmailSender,
    connection::{Connection, DBRequest, client_response::UploadResponse},
    consts::{Bt, ID},
    requests::upload::Upload,
    utils::generate_random_string,
};
use anyhow::bail;
use tokio::sync::{mpsc, oneshot};
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
        db_sender: &mpsc::Sender<DBRequest>,
        net_sender: &mpsc::Sender<Message>,
        json: &mut Upload,
    ) -> anyhow::Result<(impl Future<Output = anyhow::Result<()>>, String)> {
        let (ret_sdr, ret_rev) = oneshot::channel();
        let db_req = DBRequest::UpLoad {
            id,
            sz: Bt(json.size),
            resp: ret_sdr,
        };
        db_sender.send(db_req).await?;
        let ret = ret_rev.await?;
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
