use super::{UserInfo, generate_access_token};
use crate::{
    DbPool,
    component::EmailSender,
    entities::{prelude::*, user},
    pb::auth::authorize::v1::{AuthRequest, AuthResponse, auth_request::Account},
    server::AuthServiceProvider,
    utils,
};
use anyhow::Context;
use argon2::{PasswordHash, PasswordVerifier};
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
    #[error("db error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("Unknown Error:{0}")]
    UnknownError(#[from] anyhow::Error),
}

async fn auth_db(
    request: AuthRequest,
    db_connection: &DbPool,
) -> Result<(AuthResponse, UserInfo), AuthError> {
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
                if utils::spawn_blocking_with_tracing(move || {
                    verify_password_hash(&request.password, &passwd)
                })
                .await
                .context("computing and verifying password")?
                .is_ok()
                {
                    let token = generate_access_token(user.id.into());

                    Ok((
                        AuthResponse {
                            id: user.id as u64,
                            token,
                        },
                        UserInfo {
                            ocid: user.ocid,
                            id: user.id.into(),
                        },
                    ))
                } else {
                    Err(AuthError::WrongPassword)
                }
            }
            None => Err(AuthError::WrongPassword),
        },
        Err(e) => {
            if let DbErr::RecordNotFound(_) = e {
                Err(AuthError::UserNotFound)
            } else {
                Err(e.into())
            }
        }
    }
}

/// Login Request
pub async fn auth<T: EmailSender>(
    server: &AuthServiceProvider<T>,
    request: tonic::Request<AuthRequest>,
) -> Result<Response<AuthResponse>, Status> {
    match auth_db(request.into_inner(), &server.db).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp.0)),
        Err(e) => Err(match e {
            AuthError::WrongPassword => Status::unauthenticated(e.to_string()),
            AuthError::MissingAuthType => Status::invalid_argument(e.to_string()),
            AuthError::UserNotFound => Status::not_found(e.to_string()),
            _ => {
                tracing::error!("{}", e);
                Status::internal("Server Error")
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
