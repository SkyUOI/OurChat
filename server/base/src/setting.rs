use config::File;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod debug;

pub trait Setting {
    fn build_from_path<'de>(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: Sized,
        Self: serde::Deserialize<'de>,
    {
        let cfg = config::Config::builder()
            .add_source(File::with_name(path.as_ref().to_str().unwrap()))
            .build()?;
        let cfg = cfg.try_deserialize()?;
        Ok(cfg)
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
    pub password_strength_limit: u8,
}

impl Setting for UserSetting {}
