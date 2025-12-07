use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    #[serde(default = "crate::consts::default_tls")]
    pub enable: bool,
    #[serde(default = "crate::consts::default_client_certificate_required")]
    pub client_certificate_required: bool,
    pub server_tls_cert_path: Option<PathBuf>,
    pub server_key_cert_path: Option<PathBuf>,
    pub client_tls_cert_path: Option<PathBuf>,
    pub client_key_cert_path: Option<PathBuf>,
    pub ca_tls_cert_path: Option<PathBuf>,
    pub client_ca_tls_cert_path: Option<PathBuf>,
}

impl TlsConfig {
    pub fn is_tls_on(&self) -> anyhow::Result<bool> {
        let ret = self.enable
            && self.server_tls_cert_path.is_some()
            && self.server_key_cert_path.is_some();
        if ret {
            if !self.server_tls_cert_path.as_ref().unwrap().exists() {
                anyhow::bail!("tls_cert_path does not exist");
            }
            if !self.server_key_cert_path.as_ref().unwrap().exists() {
                anyhow::bail!("key_cert_path does not exist");
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}
