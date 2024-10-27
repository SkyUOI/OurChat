use crate::{
    client::{
        MsgConvert, requests,
        response::{ErrorMsgResponse, LoginResponse},
    },
    connection::{NetSender, UserInfo, VerifyStatus},
    entities::{prelude::*, user},
    utils,
};
use anyhow::Context;
use argon2::{PasswordHash, PasswordVerifier};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

#[derive(Debug, thiserror::Error)]
enum ErrorOfLogin {
    #[error("wrong password")]
    WrongPassword,
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),
}

pub async fn login(
    request: requests::LoginRequest,
    db_connection: &DatabaseConnection,
) -> Result<(LoginResponse, UserInfo), ErrorOfLogin> {
    // Judge login type
    let user = match request.login_type {
        requests::LoginType::Email => {
            User::find()
                .filter(user::Column::Email.eq(request.account))
                .one(db_connection)
                .await
        }
        requests::LoginType::Ocid => {
            User::find()
                .filter(user::Column::Ocid.eq(request.account))
                .one(db_connection)
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
                    match request.login_type {
                        requests::LoginType::Email => {
                            Ok((LoginResponse::success_email(user.ocid.clone()), UserInfo {
                                ocid: user.ocid,
                                id: user.id.into(),
                            }))
                        }
                        requests::LoginType::Ocid => {
                            Ok((LoginResponse::success_ocid(), UserInfo {
                                ocid: user.ocid,
                                id: user.id.into(),
                            }))
                        }
                    }
                } else {
                    Err(ErrorOfLogin::WrongPassword)
                }
            }
            None => Err(ErrorOfLogin::WrongPassword),
        },
        Err(e) => {
            if let DbErr::RecordNotFound(_) = e {
                Err(ErrorOfLogin::WrongPassword)
            } else {
                Err(e)?
            }
        }
    }
}

/// Login Request
pub async fn login_request(
    net_sender: impl NetSender,
    login_data: requests::LoginRequest,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<VerifyStatus> {
    match login(login_data, db_conn).await {
        Ok(ok_resp) => {
            net_sender.send(ok_resp.0.to_msg()).await?;
            Ok(VerifyStatus::Success(ok_resp.1))
        }
        Err(e) => {
            match e {
                ErrorOfLogin::WrongPassword => {
                    net_sender
                        .send(LoginResponse::wrong_password().to_msg())
                        .await?;
                }
                e => {
                    net_sender
                        .send(ErrorMsgResponse::server_error("Database error when log in").to_msg())
                        .await?;
                    tracing::error!("{}", e);
                }
            }
            Ok(VerifyStatus::Fail)
        }
    }
}

fn verify_password_hash(password: &str, password_hash: &str) -> anyhow::Result<()> {
    let expected = PasswordHash::new(password_hash).context("Not PHC string")?;
    argon2::Argon2::default()
        .verify_password(password.as_bytes(), &expected)
        .context("wrong password")?;
    Ok(())
}
