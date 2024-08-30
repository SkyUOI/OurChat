use crate::{
    connection::{client_response::UploadResponse, Connection},
    requests::upload::Upload,
    utils::generate_random_string,
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;

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
    pub async fn upload(
        net_sender: &mpsc::Sender<Message>,
        json: &mut Upload,
    ) -> anyhow::Result<(impl Future<Output = anyhow::Result<()>>, (String, String))> {
        let url_name = generate_url_name(&json.hash);
        let key = generate_random_string(KEY_LEN);
        let resp = UploadResponse::new(url_name.clone(), key.clone(), json.hash.clone());
        let send = async move {
            net_sender
                .send(Message::Text(serde_json::to_string(&resp).unwrap()))
                .await?;
            Ok(())
        };
        Ok((send, (url_name, key)))
    }
}
