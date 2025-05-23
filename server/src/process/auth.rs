use super::{error_msg::not_found, generate_access_token};
use crate::process::error_msg::{MISSING_AUTH_TYPE, WRONG_PASSWORD};
use crate::{
    db::helper::is_conflict, helper, process::error_msg::SERVER_ERROR, server::AuthServiceProvider,
};
use anyhow::Context;
use argon2::{PasswordHash, PasswordVerifier};
use base::database::DbPool;
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
    #[error("missing auth type")]
    MissingAuthType,
    #[error("db error:{0:?}")]
    DbError(#[from] DbErr),
    #[error("Unknown Error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
}

/// Authenticate user with the given AuthRequest and the given database connection.
///
/// Returns an AuthResponse containing the user id, the generated token, and the user's ocid,
/// together with the user info.
///
/// Errors if the user is not found, the password is wrong, or any database error occurs.
async fn auth_db(request: AuthRequest, db_connection: &DbPool) -> Result<AuthResponse, AuthError> {
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
                let passwd = user.passwd;
                if helper::spawn_blocking_with_tracing(move || {
                    verify_password_hash(&request.password, &passwd)
                })
                .await
                .context("computing and verifying password")?
                .is_ok()
                {
                    let token = generate_access_token(user.id.into());

                    Ok(AuthResponse {
                        id: user.id as u64,
                        token,
                        ocid: user.ocid.clone(),
                    })
                } else {
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
    match auth_db(request.into_inner(), &server.db).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(e) => Err(match e {
            AuthError::WrongPassword => Status::unauthenticated(WRONG_PASSWORD),
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
