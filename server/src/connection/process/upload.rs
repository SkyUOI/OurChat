use crate::{
    connection::{client_response::UploadResponse, Connection, DBRequest},
    requests::upload::Upload,
    utils::generate_random_string,
};
use tokio::sync::mpsc;
use tungstenite::Message;

const PREFIX_LEN: usize = 20;
const KEY_LEN: usize = 15;

/// 生成独一无二的url名字
/// # Details
/// 先生成20位的随机字符串，再加上图片sha256哈希值
fn generate_url_name(hash: &str) -> String {
    let prefix: String = generate_random_string(PREFIX_LEN);
    format!("{}{}", prefix, hash)
}

impl Connection {
    pub async fn upload(net_sender: &mpsc::Sender<Message>, json: Upload) -> anyhow::Result<()> {
        let url_name = generate_url_name(&json.hash);
        let resp = UploadResponse::new(url_name, generate_random_string(KEY_LEN));
        net_sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await?;
        Ok(())
    }
}
