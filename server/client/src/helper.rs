pub mod rabbitmq;

use pb::service::ourchat::download::v1::DownloadResponse;
use size::Size;
use std::env::VarError;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Once;
use std::{fmt, iter};
use tokio_stream::StreamExt;
use tonic::Streaming;

pub fn generate_file(size: Size) -> anyhow::Result<impl Iterator<Item = Vec<u8>> + Clone> {
    let size: usize = size.bytes().try_into()?;
    let ret: Vec<u8> = (0..1024_u64 * 1024_u64)
        .map(|i| (i % (u8::MAX as u64 + 1)) as u8)
        .collect();
    if size % (1024 * 1024) != 0 {
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
    format!("{:x}", hash)
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

pub const OURCHAT_TEST_CONFIG_DIR: &str = "OURCHAT_TEST_CONFIG_DIR";

/// Initialize the environment variable for testing
pub fn init_env_var() {
    // use libc-print because this function will be called in ctor function when the std::print is not available
    static TMP: Once = Once::new();
    TMP.call_once(|| {
        fn output_err(e: impl fmt::Debug) {
            libc_print::libc_eprintln!("{:?}", e);
            exit(1);
        }

        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(dotenvy::Error::Io(e)) => {
                if e.kind() != ErrorKind::NotFound {
                    output_err(e);
                }
            }
            Err(e) => {
                output_err(e);
            }
        }
        // Set config file path
        let dir = match std::env::var(OURCHAT_TEST_CONFIG_DIR) {
            Ok(d) => d,
            Err(VarError::NotPresent) => {
                libc_print::libc_eprintln!("\"{}\" is not set, please set it in \".env\" file or set this environment var directly", OURCHAT_TEST_CONFIG_DIR);
                exit(1);
            }
            Err(VarError::NotUnicode(wrong_str)) => {
                libc_print::libc_eprintln!("\"{}\" is not a valid unicode string: {}", OURCHAT_TEST_CONFIG_DIR, wrong_str.display());
                exit(1);
            }
        };
        let test_config_dir = PathBuf::from(dir);
        unsafe {
            std::env::set_var("OURCHAT_CONFIG_FILE", test_config_dir.join("ourchat.toml"));
            std::env::set_var(
                "OURCHAT_HTTP_CONFIG_FILE",
                test_config_dir.join("http.toml"),
            );
        }
    });
}
