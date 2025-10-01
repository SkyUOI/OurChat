use anyhow::{Context, anyhow};
use config::{ConfigError, File};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{path::Path, time::Duration};
use utils::merge_json;

pub mod debug;
pub mod tls;

pub trait Setting {
    fn build_from_path(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
        Self: DeserializeOwned,
    {
        read_config_and_deserialize(path)
    }
}

pub trait PathConvert {
    fn convert_to_abs_path(&mut self, full_basepath: &Path) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ContactRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "security")]
    Security,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contact {
    pub role: ContactRole,
    pub email_address: Option<email_address::EmailAddress>,
    pub phone_number: Option<String>,
    pub ocid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSetting {
    pub contacts: Vec<Contact>,
    #[serde(with = "http_serde::option::uri")]
    pub support_page: Option<http::Uri>,
    #[serde(default = "crate::consts::default_password_strength_limit")]
    pub password_strength_limit: zxcvbn::Score,
    #[serde(
        with = "humantime_serde",
        default = "crate::consts::default_verify_email_expiry"
    )]
    pub verify_email_expiry: Duration,
    #[serde(
        with = "humantime_serde",
        default = "crate::consts::default_add_friend_request_expiry"
    )]
    pub add_friend_request_expiry: Duration,
}

impl Setting for UserSetting {}

/// Read a config file from the given path
///
/// This function returns Ok(config::Config) if the file is valid, or
/// Err(ConfigError) if it is invalid.
pub fn read_a_config(path: impl AsRef<Path>) -> Result<config::Config, ConfigError> {
    config::Config::builder()
        .add_source(File::with_name(path.as_ref().to_str().unwrap()))
        .build()
}

pub fn read_config_and_deserialize<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path_ref = path.as_ref();
    let mut cfg: serde_json::Value = read_a_config(path_ref)
        .context("Failed to build config")?
        .try_deserialize()
        .context("Failed to build config")?;

    if let Some(inherit_path) = cfg.get("inherit") {
        let inherit_path =
            path_ref
                .parent()
                .context("Failed to build config")?
                .join(match inherit_path {
                    serde_json::Value::String(inherit_path) => inherit_path,
                    _ => return Err(anyhow!("Failed to build config")),
                });
        let inherit_cfg: serde_json::Value = read_config_and_deserialize(inherit_path)?;
        cfg = merge_json(inherit_cfg, cfg);
    }

    Ok(serde_json::from_value::<T>(cfg)?)
}
