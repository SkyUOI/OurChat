use super::{error_msg::not_found, generate_access_token};
use crate::db::redis::map_failed_login_to_redis;
use crate::process::error_msg::{ACCOUNT_LOCKED, MISSING_AUTH_TYPE, WRONG_PASSWORD};
use crate::{
    db::helper::is_conflict, helper, process::error_msg::SERVER_ERROR, server::AuthServiceProvider,
};
use anyhow::Context;
use argon2::{PasswordHash, PasswordVerifier};
use base::consts::ID;
use base::database::DbPool;
use deadpool_redis::redis::AsyncCommands;
use entities::{prelude::*, user};
use pb::service::auth::authorize::v1::{AuthRequest, AuthResponse, auth_request::Account};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
enum AuthError {
    #[error("user not found")]
    UserNotFound,
    #[error("wrong password")]
    WrongPassword,
    #[error("account locked")]
    AccountLocked,
    #[error("missing auth type")]
    MissingAuthType,
    #[error("db error:{0:?}")]
    DbError(#[from] DbErr),
    #[error("Unknown Error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

/// Check if the user account is locked due to too many failed login attempts.
///
/// Returns Ok if not locked, Err(AuthError::AccountLocked) if locked.
async fn check_account_locked(
    user_id: ID,
    redis_conn: &mut deadpool_redis::Connection,
    max_attempts: u32,
) -> Result<(), AuthError> {
    let key = map_failed_login_to_redis(user_id);
    let attempts: Option<u32> = redis_conn.get(&key).await?;

    if let Some(attempts) = attempts
        && attempts >= max_attempts
    {
        return Err(AuthError::AccountLocked);
    }
    Ok(())
}

/// Increment the failed login counter for a user.
async fn increment_failed_login(
    user_id: ID,
    redis_conn: &mut deadpool_redis::Connection,
    lock_duration_seconds: u64,
) -> Result<(), AuthError> {
    let key = map_failed_login_to_redis(user_id);
    let _: isize = redis_conn.incr(&key, 1).await?;

    // Set expiry on first failed attempt (when value is 1)
    let attempts: u32 = redis_conn.get(&key).await?;
    if attempts == 1 {
        let _: () = redis_conn
            .expire(&key, lock_duration_seconds as i64)
            .await?;
    }

    Ok(())
}

/// Clear failed login attempts for a user after successful login.
async fn clear_failed_login(
    user_id: ID,
    redis_conn: &mut deadpool_redis::Connection,
) -> Result<(), AuthError> {
    let key = map_failed_login_to_redis(user_id);
    let _: () = redis_conn.del(&key).await?;

    Ok(())
}

/// Authenticate user with the given AuthRequest and the given database connection.
///
/// Returns an AuthResponse containing the user id, the generated token, and the user's ocid,
/// together with the user info.
///
/// Errors if the user is not found, the password is wrong, the account is locked,
/// or any database error occurs.
async fn auth_db(
    request: AuthRequest,
    db_connection: &DbPool,
    require_email_verification: bool,
    max_failed_attempts: u32,
    lock_duration_seconds: u64,
) -> Result<AuthResponse, AuthError> {
    // Judge login type
    let login_type = match request.account {
        None => {
            return Err(AuthError::MissingAuthType);
        }
        Some(account) => account,
    };
    let user = match login_type {
        Account::Email(email) => {
            User::find()
                .filter(user::Column::Email.eq(email))
                .one(&db_connection.db_pool)
                .await
        }
        Account::Ocid(ocid) => {
            User::find()
                .filter(user::Column::Ocid.eq(ocid))
                .one(&db_connection.db_pool)
                .await
        }
    };
    match user {
        Ok(data) => match data {
            Some(user) => {
                // Check if email verification is required and if the user's email is verified
                if require_email_verification && !user.email_verified {
                    return Err(AuthError::UserNotFound); // Treat unverified users as not found
                }

                let Some(passwd) = user.passwd else {
                    return Err(AuthError::WrongPassword);
                };
                let mut redis_conn = db_connection.get_redis_connection().await?;
                // Check if account is locked before verifying password
                check_account_locked(user.id.into(), &mut redis_conn, max_failed_attempts).await?;

                if helper::spawn_blocking_with_tracing(move || {
                    verify_password_hash(&request.password, &passwd)
                })
                .await
                .context("computing and verifying password")?
                .is_ok()
                {
                    // Clear failed login attempts on successful login
                    clear_failed_login(user.id.into(), &mut redis_conn).await?;

                    let token = generate_access_token(user.id.into());

                    Ok(AuthResponse {
                        id: user.id as u64,
                        token,
                        ocid: user.ocid.clone(),
                    })
                } else {
                    // Increment failed login counter
                    increment_failed_login(user.id.into(), &mut redis_conn, lock_duration_seconds)
                        .await?;
                    Err(AuthError::WrongPassword)
                }
            }
            None => Err(AuthError::UserNotFound),
        },
        Err(e) => {
            if is_conflict(&e) {
                return Err(AuthError::UserNotFound);
            }
            Err(e.into())
        }
    }
}

/// Login Request
pub async fn auth(
    server: &AuthServiceProvider,
    request: tonic::Request<AuthRequest>,
) -> Result<Response<AuthResponse>, Status> {
    // Copy config values and drop the lock guard before await
    let (require_email_verification, max_failed_attempts, lock_duration_seconds) = {
        let cfg = server.shared_data.cfg();
        (
            cfg.main_cfg.require_email_verification,
            cfg.main_cfg.lock_account_after_failed_logins,
            cfg.main_cfg.lock_account_duration.as_secs(),
        )
    };

    match auth_db(
        request.into_inner(),
        &server.db,
        require_email_verification,
        max_failed_attempts,
        lock_duration_seconds,
    )
    .await
    {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => Err(match e {
            AuthError::WrongPassword => Status::unauthenticated(WRONG_PASSWORD),
            AuthError::AccountLocked => Status::unauthenticated(ACCOUNT_LOCKED),
            AuthError::MissingAuthType => Status::invalid_argument(MISSING_AUTH_TYPE),
            AuthError::UserNotFound => Status::not_found(not_found::USER),
            _ => {
                tracing::error!("{}", e);
                Status::internal(SERVER_ERROR)
            }
        }),
    }
}

fn verify_password_hash(password: &str, password_hash: &str) -> anyhow::Result<()> {
    let expected = PasswordHash::new(password_hash).context("Not PHC string")?;
    argon2::Argon2::default()
        .verify_password(password.as_bytes(), &expected)
        .context("wrong password")?;
    Ok(())
}
