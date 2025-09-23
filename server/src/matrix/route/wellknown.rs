use std::sync::Arc;

use crate::SharedData;
use crate::matrix::defines::MatrixUserId;
use axum::{Json, extract::State, routing::get};
use base::consts::OCID;
use base::setting;
use http::Uri;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ContactRole {
    #[serde(rename = "m.role.admin")]
    Admin,
    #[serde(rename = "m.role.security")]
    Security,
}

impl From<setting::ContactRole> for ContactRole {
    fn from(value: setting::ContactRole) -> Self {
        match value {
            setting::ContactRole::Admin => Self::Admin,
            setting::ContactRole::Security => Self::Security,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub email_address: Option<email_address::EmailAddress>,
    pub matrix_id: Option<MatrixUserId>,
    pub role: ContactRole,
}

impl Contact {
    fn try_from(value: setting::Contact, domain: impl AsRef<str>) -> Self {
        Self {
            email_address: value.email_address,
            role: value.role.into(),
            matrix_id: value
                .ocid
                .map(|x| MatrixUserId::from_ocid(&OCID(x), domain)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportResponse {
    pub contacts: Vec<Contact>,
    #[serde(with = "http_serde::option::uri")]
    pub support_page: Option<Uri>,
}

pub async fn support(State(cfg): State<Arc<SharedData>>) -> Json<SupportResponse> {
    let contacts = cfg
        .cfg
        .user_setting
        .contacts
        .iter()
        .map(|x| Contact::try_from(x.clone(), cfg.cfg.http_cfg.domain()))
        .collect();
    let response = SupportResponse {
        contacts,
        support_page: cfg.cfg.user_setting.support_page.clone(),
    };
    Json(response)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HomeserverInfo {
    #[serde(with = "http_serde::uri")]
    pub base_url: Uri,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityServerInfo {
    #[serde(with = "http_serde::uri")]
    pub base_url: Uri,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientResponse {
    #[serde(rename = "m.homeserver")]
    pub m_homeserver: HomeserverInfo,
    #[serde(rename = "m.identity_server")]
    pub m_identity_server: Option<IdentityServerInfo>,
}

async fn client(State(cfg): State<Arc<SharedData>>) -> Json<ClientResponse> {
    let ret = ClientResponse {
        m_homeserver: HomeserverInfo {
            base_url: cfg.cfg.http_cfg.base_url().clone(),
        },
        m_identity_server: None,
    };
    Json(ret)
}

pub fn configure_route() -> axum::Router<Arc<SharedData>> {
    axum::Router::new()
        .route("/matrix/support", get(support))
        .route("/matrix/client", get(client))
}
