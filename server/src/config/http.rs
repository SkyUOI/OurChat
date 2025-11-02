use std::{path::PathBuf, time::Duration};

use base::setting::tls::TlsConfig;
use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, derive::PathConvert)]
pub struct HttpCfg {
    #[serde(default = "base::consts::default_ip")]
    pub ip: String,
    #[serde(default = "base::consts::default_port")]
    pub port: u16,
    pub logo_path: PathBuf,
    pub verification_html_template_path: Option<PathBuf>,
    pub default_avatar_path: PathBuf,
    #[serde(default = "base::consts::default_http_run_migration")]
    pub run_migration: bool,
    #[serde(default = "base::consts::default_enable_matrix")]
    pub enable_matrix: bool,
    #[serde(
        default = "base::consts::default_log_clean_duration",
        with = "humantime_serde"
    )]
    pub log_clean_duration: Duration,
    #[serde(default = "base::consts::default_log_keep", with = "humantime_serde")]
    pub lop_keep: Duration,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub rate_limit: RateLimitCfg,
    pub email_cfg: Option<PathBuf>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RateLimitCfg {
    #[serde(default = "base::consts::default_rate_limit_enable")]
    pub enable: bool,
    #[serde(default = "base::consts::default_rate_limit_burst")]
    pub num_of_burst_requests: u32,
    #[serde(
        default = "base::consts::default_rate_limit_replenish_duration",
        with = "humantime_serde"
    )]
    pub replenish_duration: Duration,
}

impl Default for RateLimitCfg {
    fn default() -> Self {
        let empty = serde_json::json!({});
        serde_json::from_value(empty).unwrap()
    }
}

impl base::setting::Setting for HttpCfg {}

impl HttpCfg {
    pub fn protocol_http(&self) -> &'static str {
        if self.tls.enable { "https" } else { "http" }
    }

    pub fn base_url(&self) -> Uri {
        format!("{}://{}:{}", self.protocol_http(), self.ip, self.port)
            .parse()
            .unwrap()
    }

    pub fn domain(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}