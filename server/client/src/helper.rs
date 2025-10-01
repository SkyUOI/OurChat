pub mod rabbitmq;

use pb::service::ourchat::download::v1::DownloadResponse;
use size::Size;
use std::iter;
use tokio_stream::StreamExt;
use tonic::Streaming;

pub fn generate_file(size: Size) -> anyhow::Result<impl Iterator<Item = Vec<u8>> + Clone> {
    let size: usize = size.bytes().try_into()?;
    let ret: Vec<u8> = (0..1024_u64 * 1024_u64)
        .map(|i| (i % (u8::MAX as u64 + 1)) as u8)
        .collect();
    if !size.is_multiple_of(1024 * 1024) {
        Ok(iter::repeat_n(ret.clone(), size / 1024 / 1024)
            .chain(iter::once(ret[..size % (1024 * 1024)].to_vec())))
    } else {
        Ok(iter::repeat_n(ret, size / 1024 / 1024).chain(iter::once(vec![])))
    }
}

pub fn get_hash_from_file(content: impl Iterator<Item = Vec<u8>> + Clone) -> String {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    for chunks in content {
        hasher.update(&chunks);
    }
    let hash = hasher.finalize();
    format!("{hash:x}")
}

pub async fn get_hash_from_download(
    mut content: Streaming<DownloadResponse>,
) -> anyhow::Result<String> {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    while let Some(stream) = content.next().await {
        let stream = stream?;
        hasher.update(stream.data);
    }
    let hash = format!("{:x}", hasher.finalize());
    Ok(hash)
}
