use super::error_msg::{NOT_STRONG_PASSWORD, invalid};
use super::generate_access_token;
use crate::process::error_msg::{SERVER_ERROR, exist};
use crate::{db, helper, server::AuthServiceProvider, shared_state};
use anyhow::Context;
use argon2::{Params, PasswordHasher, password_hash::SaltString};
use base::consts::{self, ID};
use base::database::DbPool;
use entities::user;
use migration::m20220101_000001_create_table::USERNAME_MAX_LEN;
use pb::service::auth::register::v1::{RegisterRequest, RegisterResponse};
use rand::rngs::OsRng;
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
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
        friend_limit: ActiveValue::Set(shared_state::get_friends_number_limit().try_into()?),
        public_key: ActiveValue::Set(request.public_key.into()),
        ..Default::default()
    };
    match user.insert(&db_connection.db_pool).await {
        Ok(res) => {
            // Happy Path
            let response = RegisterResponse {
                id: res.id as u64,
                token: generate_access_token(id),
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
    let salt = SaltString::try_from_rng(&mut OsRng)?;

    Ok(
        argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
            .hash_password(password.as_bytes(), &salt)?
            .to_string(),
    )
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
        < server.shared_data.cfg.user_setting.password_strength_limit
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

    let password_hash = &server.shared_data.cfg.main_cfg.password_hash;
    let response = add_new_user(
        req,
        &server.db,
        Params::new(
            password_hash.m_cost,
            password_hash.t_cost,
            password_hash.p_cost,
            password_hash.output_len,
        )
        .unwrap(),
    )
    .await?;
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
        },
    }
}
