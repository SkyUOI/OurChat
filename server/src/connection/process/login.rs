use crate::{
    client::{
        requests,
        response::{self, LoginResponse},
    },
    connection::{NetSender, VerifyStatus},
    consts::ID,
    utils,
};
use anyhow::Context;
use argon2::{PasswordHash, PasswordVerifier};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};

#[derive::db_compatibility]
pub async fn login(
    request: requests::Login,
    db_connection: &DatabaseConnection,
) -> anyhow::Result<Result<(LoginResponse, ID), requests::Status>> {
    use entities::prelude::*;
    use entities::user::Column;
    use requests::Status;
    use response::login::Status;
    // Judge login type
    let user = match request.login_type {
        requests::LoginType::Email => {
            User::find()
                .filter(Column::Email.eq(request.account))
                .one(db_connection)
                .await
        }
        requests::LoginType::Ocid => {
            User::find()
                .filter(Column::Ocid.eq(request.account))
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
                        requests::LoginType::Email => Ok(Ok((
                            LoginResponse::success_email(user.ocid),
                            user.id.into(),
                        ))),
                        requests::LoginType::Ocid => {
                            Ok(Ok((LoginResponse::success_ocid(), user.id.into())))
                        }
                    }
                } else {
                    Ok(Err(Status!(WrongPassword)))
                }
            }
            None => Ok(Err(Status!(WrongPassword))),
        },
        Err(e) => {
            if let DbErr::RecordNotFound(_) = e {
                Ok(Err(Status!(WrongPassword)))
            } else {
                tracing::error!("database error:{}", e);
                Ok(Err(Status::ServerError))
            }
        }
    }
}

/// Login Request
pub async fn login_request(
    net_sender: impl NetSender,
    login_data: requests::Login,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<VerifyStatus> {
    match login(login_data, db_conn).await? {
        Ok(ok_resp) => {
            net_sender.send(ok_resp.0.into()).await?;
            Ok(VerifyStatus::Success(ok_resp.1))
        }
        Err(e) => {
            net_sender.send(LoginResponse::failed(e).into()).await?;
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
