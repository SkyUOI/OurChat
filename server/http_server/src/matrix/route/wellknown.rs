use crate::Cfg;
use crate::matrix::defines::MatrixUserId;
use actix_web::{HttpResponse, Responder, get, web};
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
    #[serde(with = "http_serde::uri")]
    pub support_page: Uri,
}

#[get("/matrix/support")]
pub async fn support(cfg: web::Data<Cfg>) -> impl Responder {
    let contacts = cfg
        .user_setting
        .contacts
        .iter()
        .map(|x| Contact::try_from(x.clone(), cfg.main_cfg.domain()))
        .collect();
    let response = SupportResponse {
        contacts,
        support_page: cfg.user_setting.support_page.clone(),
    };
    HttpResponse::Ok().json(response)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HomeserverInfo {
    #[serde(with = "http_serde::uri")]
    base_url: Uri,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityServerInfo {
    #[serde(with = "http_serde::uri")]
    base_url: Uri,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientResponse {
    #[serde(rename = "m.homeserver")]
    m_homeserver: HomeserverInfo,
    #[serde(rename = "m.identity_server")]
    m_identity_server: Option<IdentityServerInfo>,
}

#[get("/matrix/client")]
async fn client(cfg: web::Data<Cfg>) -> impl Responder {
    let ret = ClientResponse {
        m_homeserver: HomeserverInfo {
            base_url: cfg.main_cfg.base_url().clone(),
        },
        m_identity_server: None,
    };
    HttpResponse::Ok().json(ret)
}

pub fn configure_route(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/.well-known").service(support).service(client);
    cfg.service(scope);
}
