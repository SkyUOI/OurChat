use super::error_msg::{NOT_STRONG_PASSWORD, invalid};
use super::generate_access_token;
use crate::db::session::join_in_session_or_create;
use crate::process::error_msg::{SERVER_ERROR, exist};
use crate::{db, helper, server::AuthServiceProvider};
use anyhow::Context;
use argon2::{Params, PasswordHasher};
use base::consts::{self, ID};
use base::database::DbPool;
use entities::user;
use migration::constants::USERNAME_MAX_LEN;
use pb::service::auth::register::v1::{RegisterRequest, RegisterResponse};
use rsa::RsaPublicKey;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
// use rand::rngs::OsRng;
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr, TransactionTrait};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::num::TryFromIntError;
use tonic::{Request, Response, Status};
use tracing::error;

/// Register a new user
///
/// The function takes a `RegisterRequest`, a database connection, and an argon2 parameters
/// object. It generates a snowflake id and a random ocid, computes the password hash, and
/// inserts the user into the database. If the insertion fails due to a conflict (i.e., the
/// user already exists), it returns a `RegisterError::UserExists` error. Otherwise, it
/// returns a `RegisterResponse` containing the id, the generated token, and the ocid.
///
/// # Errors
///
/// - `RegisterError::UserExists`: The user already exists in the database.
/// - `RegisterError::InternalError`: An internal error occurred.
async fn add_new_user(
    request: RegisterRequest,
    db_connection: &DbPool,
    params: Params,
    require_email_verification: bool,
    friends_number_limit: u32,
) -> Result<RegisterResponse, RegisterError> {
    // Generate snowflake id
    let id = ID(helper::USER_ID_GENERATOR
        .generate()
        .context("failed to generate snowflake id")?
        .into_i64()
        .try_into()?);
    // Generate ocid by random
    let ocid = helper::generate_ocid(consts::OCID_LEN);
    let passwd = request.password;
    let passwd =
        helper::spawn_blocking_with_tracing(move || compute_password_hash(&passwd, params))
            .await
            .context("compute hash async task error")??;
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ocid: ActiveValue::Set(ocid),
        passwd: ActiveValue::Set(Some(passwd)),
        name: ActiveValue::Set(request.name),
        email: ActiveValue::Set(request.email),
        time: ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: ActiveValue::Set(0),
        friends_num: ActiveValue::Set(0),
        friend_limit: ActiveValue::Set(friends_number_limit.try_into()?),
        public_key: ActiveValue::Set(request.public_key.into()),
        email_verified: ActiveValue::Set(!require_email_verification),
        ..Default::default()
    };
    match user.insert(&db_connection.db_pool).await {
        Ok(res) => {
            // Happy Path
            let response = RegisterResponse {
                id: res.id as u64,
                token: generate_access_token(id)
                    .with_context(|| format!("Couldn't generate jwt for {}", id))?,
                ocid: res.ocid.clone(),
            };
            Ok(response)
        }
        Err(e) => {
            if db::helper::is_conflict(&e) {
                return Err(RegisterError::UserExists);
            }
            Err(e.into())
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum RegisterError {
    #[error("User exists")]
    UserExists,
    #[error("database error:{0:?}")]
    DbError(#[from] DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
    #[error("from int error")]
    FromIntError(#[from] TryFromIntError),
    #[error("Password not Strong")]
    PasswordNotStrong,
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("Invalid username format")]
    InvalidUsername,
    #[error("Invalid public key")]
    InvalidPublicKey,
}

/// Computes the password hash using the given argon2 parameters
///
/// It generates a random salt, and uses the given parameters to compute the hash.
/// The function returns the computed hash as a string.
///
/// # Panics
///
/// Panics if the password is too long or if the salt generation fails.
fn compute_password_hash(password: &str, params: Params) -> anyhow::Result<String> {
    Ok(
        argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
            .hash_password(password.as_bytes())?
            .to_string(),
    )
}

/// Validates the RSA public key format and size.
///
/// Checks that the key is a valid RSA public key in PKCS#1 DER format
/// and that the key size is at least 2048 bits for security.
fn validate_public_key(public_key: &[u8]) -> Result<(), RegisterError> {
    // Check minimum length (an empty or very short key is definitely invalid)
    if public_key.is_empty() || public_key.len() < 32 {
        return Err(RegisterError::InvalidPublicKey);
    }

    // Try to parse
    let key = RsaPublicKey::from_public_key_der(public_key)
        .ok() // PKCS#8
        .or_else(|| RsaPublicKey::from_pkcs1_der(public_key).ok()) // PKCS#1
        .or_else(|| {
            // Try PEM (PKCS#8)
            std::str::from_utf8(public_key)
                .ok()
                .and_then(|pem| RsaPublicKey::from_public_key_pem(pem).ok())
        })
        .ok_or(RegisterError::InvalidPublicKey)?;

    // Check key size - minimum 2048 bits for security (256 bytes)
    // Maximum 8192 bits for practical reasons
    let key_size = key.size();
    const MIN_KEY_BITS: usize = 2048;
    const MAX_KEY_BITS: usize = 8192;

    if key_size * 8 < MIN_KEY_BITS || key_size * 8 > MAX_KEY_BITS {
        return Err(RegisterError::InvalidPublicKey);
    }

    Ok(())
}

/// Internal implementation of the register process
///
/// Checks the strength of the password, validity of the email and username
/// and generates a new user if all checks pass.
///
/// # Errors
///
/// Returns a `RegisterError` if the password is not strong enough,
/// if the email is invalid, or if the username is invalid.
///
async fn register_impl(
    server: &AuthServiceProvider,
    request: Request<RegisterRequest>,
) -> Result<RegisterResponse, RegisterError> {
    let req = request.into_inner();

    // Check strong password
    if zxcvbn::zxcvbn(&req.password, &[&req.name, &req.email]).score()
        < server
            .shared_data
            .cfg()
            .user_setting
            .password_strength_limit
    {
        return Err(RegisterError::PasswordNotStrong);
    }

    // Check if email is valid
    if !email_address::EmailAddress::is_valid(&req.email) {
        return Err(RegisterError::InvalidEmail);
    }

    // User Name Check
    if req.name.trim().is_empty() || req.name.len() > USERNAME_MAX_LEN {
        return Err(RegisterError::InvalidUsername);
    }

    // Validate public key format and size
    validate_public_key(&req.public_key)?;

    let params = Params::new(
        server.shared_data.cfg().main_cfg.password_hash.m_cost,
        server.shared_data.cfg().main_cfg.password_hash.t_cost,
        server.shared_data.cfg().main_cfg.password_hash.p_cost,
        server.shared_data.cfg().main_cfg.password_hash.output_len,
    )
    .context("Invalid Argon2 parameters - check password_hash configuration")?;
    let require_email_verification = server.shared_data.cfg().main_cfg.require_email_verification;
    let friends_number_limit = server.shared_data.cfg().main_cfg.friends_number_limit;
    let response = add_new_user(
        req,
        &server.db,
        params,
        require_email_verification,
        friends_number_limit,
    )
    .await?;
    // join default session if configured
    let default_session = server.shared_data.cfg().main_cfg.default_session;
    if let Some(default_session_id) = default_session {
        let logic = async {
            let transaction = server.db.db_pool.begin().await?;
            if let Err(e) = join_in_session_or_create(
                default_session_id,
                ID(response.id),
                None,
                &transaction,
                false,
            )
            .await
            {
                transaction.rollback().await?;
                return Err(e.into());
            }
            transaction.commit().await?;
            anyhow::Ok(())
        };
        // default session join is optional and failure is acceptable, so register still succeeds
        if let Err(e) = logic.await {
            error!(
                "failed to join default session for new user {}: {:?}",
                response.id, e
            );
        }
    }
    Ok(response)
}

pub async fn register(
    server: &AuthServiceProvider,
    request: Request<RegisterRequest>,
) -> Result<Response<RegisterResponse>, Status> {
    match register_impl(server, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => match e {
            RegisterError::UserExists => Err(Status::already_exists(exist::USER)),
            RegisterError::DbError(_)
            | RegisterError::UnknownError(_)
            | RegisterError::FromIntError(_) => {
                error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            RegisterError::PasswordNotStrong => Err(Status::invalid_argument(NOT_STRONG_PASSWORD)),
            RegisterError::InvalidEmail => Err(Status::invalid_argument(invalid::EMAIL_ADDRESS)),
            RegisterError::InvalidUsername => Err(Status::invalid_argument(invalid::USERNAME)),
            RegisterError::InvalidPublicKey => Err(Status::invalid_argument(invalid::PUBLIC_KEY)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1::EncodeRsaPublicKey;
    use rsa::pkcs8::EncodePublicKey;

    #[test]
    fn test_validate_public_key_empty() {
        let result = validate_public_key(&[]);
        assert!(matches!(result, Err(RegisterError::InvalidPublicKey)));
    }

    #[test]
    fn test_validate_public_key_too_short() {
        let result = validate_public_key(&[0u8; 16]);
        assert!(matches!(result, Err(RegisterError::InvalidPublicKey)));
    }

    #[test]
    fn test_validate_public_key_invalid_format() {
        // Invalid DER format (just repeated bytes)
        let result = validate_public_key(&[0xDE; 300]);
        assert!(matches!(result, Err(RegisterError::InvalidPublicKey)));
    }

    #[test]
    fn test_validate_public_key_1024_bits_too_small() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 1024).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let key_bytes = public_key.to_pkcs1_der().unwrap().as_bytes().to_vec();

        let result = validate_public_key(&key_bytes);
        assert!(matches!(result, Err(RegisterError::InvalidPublicKey)));
    }

    // PKCS#1 DER format tests
    #[test]
    fn test_validate_public_key_pkcs1_der_2048_bits_valid() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let key_bytes = public_key.to_pkcs1_der().unwrap().as_bytes().to_vec();

        let result = validate_public_key(&key_bytes);
        assert!(result.is_ok());
    }

    // PKCS#8 DER format tests
    #[test]
    fn test_validate_public_key_pkcs8_der_2048_bits_valid() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let key_bytes = public_key.to_public_key_der().unwrap().as_bytes().to_vec();

        let result = validate_public_key(&key_bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_public_key_pkcs8_der_4096_bits_valid() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 4096).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let key_bytes = public_key.to_public_key_der().unwrap().as_bytes().to_vec();

        let result = validate_public_key(&key_bytes);
        assert!(result.is_ok());
    }

    // PEM format tests (PKCS#8)
    #[test]
    fn test_validate_public_key_pem_2048_bits_valid() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let pem_str = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let key_bytes = pem_str.into_bytes();

        let result = validate_public_key(&key_bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_public_key_pem_4096_bits_valid() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 4096).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let pem_str = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let key_bytes = pem_str.into_bytes();

        let result = validate_public_key(&key_bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_public_key_pem_1024_bits_too_small() {
        let mut rng = rand::rng();
        let private_key = RsaPrivateKey::new(&mut rng, 1024).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        let pem_str = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let key_bytes = pem_str.into_bytes();

        let result = validate_public_key(&key_bytes);
        assert!(matches!(result, Err(RegisterError::InvalidPublicKey)));
    }
}
