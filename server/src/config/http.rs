use std::{path::PathBuf, time::Duration};

use anyhow::bail;
use base::setting::tls::TlsConfig;
use http::Uri;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use utils::serde_default;

#[derive(Debug, Serialize, Clone, derive::PathConvert)]
#[serde(deny_unknown_fields)]
pub struct HttpCfg {
    pub inherit: Option<PathBuf>,
    pub ip: String,
    pub port: u16,
    pub logo_path: PathBuf,
    pub verification_html_template_path: Option<PathBuf>,
    pub default_avatar_path: PathBuf,
    pub run_migration: bool,
    pub enable_matrix: bool,
    pub log_clean_duration: Duration,
    pub lop_keep: Duration,
    pub tls: TlsConfig,
    pub rate_limit: RateLimitCfg,
    #[path_convert]
    pub web_panel: WebPanelCfg,
    pub email_cfg: Option<PathBuf>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
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

serde_default!(RateLimitCfg);

#[derive(Deserialize, Serialize, Clone, Debug, derive::PathConvert)]
#[serde(deny_unknown_fields)]
pub struct WebPanelCfg {
    #[serde(default = "base::consts::default_web_panel_enable")]
    pub enable: bool,
    #[serde(default = "base::consts::default_web_panel_dist_path")]
    pub dist_path: PathBuf,
}

serde_default!(WebPanelCfg);

/// Raw struct for deserialization with all serde attributes
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawHttpCfg {
    #[serde(default)]
    pub inherit: Option<PathBuf>,
    #[serde(default = "base::consts::default_ip")]
    pub ip: String,
    #[serde(default = "base::consts::default_port")]
    pub port: u16,
    pub logo_path: PathBuf,
    #[serde(default)]
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
    #[serde(default)]
    pub web_panel: WebPanelCfg,
    #[serde(default)]
    pub email_cfg: Option<PathBuf>,
}

impl<'de> Deserialize<'de> for HttpCfg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawHttpCfg::deserialize(deserializer)?;

        // Validate configuration
        if raw.port == 0 {
            return Err(D::Error::custom("port must be greater than 0"));
        }
        if raw.log_clean_duration.is_zero() {
            return Err(D::Error::custom("log_clean_duration cannot be zero"));
        }
        if raw.lop_keep.is_zero() {
            return Err(D::Error::custom("lop_keep cannot be zero"));
        }

        Ok(HttpCfg {
            inherit: raw.inherit,
            ip: raw.ip,
            port: raw.port,
            logo_path: raw.logo_path,
            verification_html_template_path: raw.verification_html_template_path,
            default_avatar_path: raw.default_avatar_path,
            run_migration: raw.run_migration,
            enable_matrix: raw.enable_matrix,
            log_clean_duration: raw.log_clean_duration,
            lop_keep: raw.lop_keep,
            tls: raw.tls,
            rate_limit: raw.rate_limit,
            web_panel: raw.web_panel,
            email_cfg: raw.email_cfg,
        })
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
            .expect("Invalid ip or port")
    }

    pub fn domain(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// Validates that all required file paths exist
    pub fn validate_paths(&self) -> anyhow::Result<()> {
        // Required paths
        if !self.logo_path.exists() {
            bail!("logo_path does not exist: {}", self.logo_path.display());
        }
        if !self.default_avatar_path.exists() {
            bail!(
                "default_avatar_path does not exist: {}",
                self.default_avatar_path.display()
            );
        }

        // Optional paths
        if let Some(ref template_path) = self.verification_html_template_path
            && !template_path.exists()
        {
            bail!(
                "verification_html_template_path does not exist: {}",
                template_path.display()
            );
        }

        if let Some(ref email_cfg) = self.email_cfg
            && !email_cfg.exists()
        {
            bail!("email_cfg does not exist: {}", email_cfg.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::File;

    /// Helper to create a minimal valid config JSON
    fn minimal_valid_config() -> serde_json::Value {
        json!({
            "ip": "127.0.0.1",
            "port": 8080,
            "logo_path": "/tmp/logo.png",
            "default_avatar_path": "/tmp/avatar.png",
            "log_clean_duration": "1d",
            "lop_keep": "7d"
        })
    }

    #[test]
    fn test_valid_config_deserializes() {
        let config = minimal_valid_config();
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(
            result.is_ok(),
            "Valid config should deserialize successfully"
        );
        let cfg = result.unwrap();
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn test_port_zero_fails() {
        let mut config = minimal_valid_config();
        config["port"] = json!(0);
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("port must be greater than 0"),
            "Error was: {}",
            err
        );
    }

    #[test]
    fn test_log_clean_duration_zero_fails() {
        let mut config = minimal_valid_config();
        config["log_clean_duration"] = json!("0s");
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("log_clean_duration cannot be zero"));
    }

    #[test]
    fn test_lop_keep_zero_fails() {
        let mut config = minimal_valid_config();
        config["lop_keep"] = json!("0s");
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("lop_keep cannot be zero"));
    }

    #[test]
    fn test_multiple_validation_errors_first_is_returned() {
        let mut config = minimal_valid_config();
        config["port"] = json!(0);
        config["log_clean_duration"] = json!("0s");
        config["lop_keep"] = json!("0s");
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_err());
        // Should fail on first validation error
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("port must be greater than 0"),
            "Error was: {}",
            err
        );
    }

    #[test]
    fn test_validate_logo_path_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let avatar_path = temp_dir.path().join("avatar.png");
        File::create(&avatar_path).unwrap();

        let mut config = minimal_valid_config();
        config["logo_path"] = json!("/nonexistent/path/logo.png");
        config["default_avatar_path"] = json!(avatar_path);
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok()); // Deserialization succeeds

        let cfg = result.unwrap();
        assert!(cfg.validate_paths().is_err());
        let err = cfg.validate_paths().unwrap_err().to_string();
        assert!(err.contains("logo_path does not exist"));
    }

    #[test]
    fn test_validate_default_avatar_path_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let logo_path = temp_dir.path().join("logo.png");
        File::create(&logo_path).unwrap();

        let mut config = minimal_valid_config();
        config["logo_path"] = json!(logo_path);
        config["default_avatar_path"] = json!("/nonexistent/path/avatar.png");
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok()); // Deserialization succeeds

        let cfg = result.unwrap();
        assert!(cfg.validate_paths().is_err());
        let err = cfg.validate_paths().unwrap_err().to_string();
        assert!(err.contains("default_avatar_path does not exist"));
    }

    #[test]
    fn test_validate_verification_html_template_path_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let logo_path = temp_dir.path().join("logo.png");
        let avatar_path = temp_dir.path().join("avatar.png");
        File::create(&logo_path).unwrap();
        File::create(&avatar_path).unwrap();

        let mut config = minimal_valid_config();
        config["logo_path"] = json!(logo_path);
        config["default_avatar_path"] = json!(avatar_path);
        config["verification_html_template_path"] = json!("/nonexistent/path/template.html");
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok()); // Deserialization succeeds

        let cfg = result.unwrap();
        assert!(cfg.validate_paths().is_err());
        let err = cfg.validate_paths().unwrap_err().to_string();
        assert!(
            err.contains("verification_html_template_path does not exist"),
            "Error was: {}",
            err
        );
    }

    #[test]
    fn test_validate_email_cfg_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let logo_path = temp_dir.path().join("logo.png");
        let avatar_path = temp_dir.path().join("avatar.png");
        File::create(&logo_path).unwrap();
        File::create(&avatar_path).unwrap();

        let mut config = minimal_valid_config();
        config["logo_path"] = json!(logo_path);
        config["default_avatar_path"] = json!(avatar_path);
        config["email_cfg"] = json!("/nonexistent/path/email.toml");
        let result: Result<HttpCfg, _> = serde_json::from_value(config);
        assert!(result.is_ok()); // Deserialization succeeds

        let cfg = result.unwrap();
        assert!(cfg.validate_paths().is_err());
        let err = cfg.validate_paths().unwrap_err().to_string();
        assert!(err.contains("email_cfg does not exist"));
    }
}
