use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default = "crate::consts::default_tls")]
    pub enable: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub key_cert_path: Option<PathBuf>,
}

impl TlsConfig {
    pub fn is_tls_on(&self) -> anyhow::Result<bool> {
        let ret = self.enable && self.tls_cert_path.is_some() && self.key_cert_path.is_some();
        if ret {
            if !self.tls_cert_path.as_ref().unwrap().exists() {
                anyhow::bail!("tls_cert_path does not exist");
            }
            if !self.key_cert_path.as_ref().unwrap().exists() {
                anyhow::bail!("key_cert_path does not exist");
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}