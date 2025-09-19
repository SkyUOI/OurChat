use anyhow::Context;
use config::{ConfigError, File};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

pub mod debug;
pub mod tls;

pub trait Setting {
    fn build_from_path<'de>(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
        Self: serde::Deserialize<'de>,
    {
        Ok(read_config_and_deserialize(path)?)
    }
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

pub fn read_config_and_deserialize<'de, T: Deserialize<'de>>(
    path: impl AsRef<Path>,
) -> anyhow::Result<T> {
    read_a_config(path)
        .context("Failed to build config")?
        .try_deserialize()
        .context("Failed to build config")
}
