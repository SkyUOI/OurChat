use super::{generate_access_token, wrong_password};
use crate::{
    DbPool,
    component::EmailSender,
    connection::UserInfo,
    entities::{prelude::*, user},
    pb::login::{LoginRequest, LoginResponse, login_request::Account},
    server::{AuthServiceProvider, RpcServer},
    utils,
};
use anyhow::Context;
use argon2::{PasswordHash, PasswordVerifier};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use tonic::{Response, Status};

async fn login_db(
    request: LoginRequest,
    db_connection: &DbPool,
) -> Result<(LoginResponse, UserInfo), tonic::Status> {
    // Judge login type
    let login_type = match request.account {
        None => {
            return Err(Status::invalid_argument("missing account"));
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
                .is_ok()
                {
                    let token = generate_access_token(user.id.into());

                    Ok((
                        LoginResponse {
                            ocid: user.ocid.clone(),
                            token,
                        },
                        UserInfo {
                            ocid: user.ocid,
                            id: user.id.into(),
                        },
                    ))
                } else {
                    Err(wrong_password())
                }
            }
            None => Err(wrong_password()),
        },
        Err(e) => {
            if let DbErr::RecordNotFound(_) = e {
                Err(wrong_password())
            } else {
                Err(tonic::Status::internal("database error"))
            }
        }
    }
}

/// Login Request
pub async fn login<T: EmailSender>(
    server: &AuthServiceProvider<T>,
    request: tonic::Request<LoginRequest>,
) -> Result<Response<LoginResponse>, Status> {
    match login_db(request.into_inner(), &server.db).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp.0)),
        Err(e) => Err(e),
    }
}

fn verify_password_hash(password: &str, password_hash: &str) -> anyhow::Result<()> {
    let expected = PasswordHash::new(password_hash).context("Not PHC string")?;
    argon2::Argon2::default()
        .verify_password(password.as_bytes(), &expected)
        .context("wrong password")?;
    Ok(())
}
