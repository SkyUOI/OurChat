//! Functions process the requests from clients

pub mod get_account_info;
mod get_user_msg;
pub mod login;
pub mod new_session;
pub mod register;
mod send_msg;
mod set_account_info;
mod set_friend_info;
pub mod unregister;
mod upload;
pub mod verify;

pub use get_user_msg::get_user_msg;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
pub use new_session::new_session;
pub use send_msg::send_msg;
use serde::Deserialize;
use serde::Serialize;
pub use set_account_info::set_account_info;
pub use set_friend_info::set_friend_info;
use std::time::Duration;
use tonic::Request;
use tonic::Status;
pub use unregister::unregister;
pub use upload::upload;

use crate::SERVER_INFO;
use crate::consts::ID;

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTdata {
    pub id: ID,
    exp: i64,
}

fn wrong_password() -> tonic::Status {
    Status::unauthenticated("wrong password")
}

const EXPIRE_TIME: Duration = Duration::from_days(5);

pub fn generate_access_token(id: ID) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &JWTdata {
            id,
            exp: chrono::Utc::now().timestamp() + EXPIRE_TIME.as_secs() as i64,
        },
        &EncodingKey::from_secret(SERVER_INFO.secret.as_bytes()),
    )
    .unwrap()
}

pub fn check_token(token: &str) -> Option<JWTdata> {
    match access_token(token) {
        Ok(data) => {
            if chrono::offset::Utc::now().timestamp() < data.exp {
                Some(data)
            } else {
                None
            }
        }
        Err(_) => {
            tracing::trace!("jwt format wrong");
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrAuth {
    #[error("Expire")]
    Expire,
    #[error("JWT error")]
    JWT(#[from] jsonwebtoken::errors::Error),
}

pub fn access_token(token: &str) -> Result<JWTdata, ErrAuth> {
    let token = jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(SERVER_INFO.secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token.claims)
}

pub fn get_id_from_req<T>(req: &Request<T>) -> Option<ID> {
    req.metadata()
        .get("id")
        .map(|id| ID(id.to_str().unwrap().parse::<u64>().unwrap()))
}
