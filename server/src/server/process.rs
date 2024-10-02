use super::Server;
use crate::{
    connection::client_response::{self, LoginResponse, NewSessionResponse, RegisterResponse},
    consts::{self, Bt, ID},
    requests, shared_state, utils,
};
use anyhow::Context;
use argon2::{Params, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use snowdon::ClassicLayoutSnowflakeExtension;
use tokio::sync::oneshot;

impl Server {
    #[derive::db_compatibility]
    pub async fn login(
        request: requests::Login,
        resp: oneshot::Sender<Result<(LoginResponse, ID), requests::Status>>,
        db_connection: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        use client_response::login::Status;
        use entities::prelude::*;
        use entities::user::Column;
        use requests::Status;
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
                            requests::LoginType::Email => resp
                                .send(Ok((
                                    LoginResponse::success_email(user.ocid),
                                    user.id.into(),
                                )))
                                .unwrap(),
                            requests::LoginType::Ocid => resp
                                .send(Ok((LoginResponse::success_ocid(), user.id.into())))
                                .unwrap(),
                        }
                    } else {
                        resp.send(Err(Status!(WrongPassword))).unwrap();
                    }
                }
                None => resp.send(Err(Status!(WrongPassword))).unwrap(),
            },
            Err(e) => {
                if let DbErr::RecordNotFound(_) = e {
                    resp.send(Err(Status!(WrongPassword))).unwrap();
                } else {
                    tracing::error!("database error:{}", e);
                    resp.send(Err(Status::ServerError)).unwrap();
                }
            }
        }
        Ok(())
    }

    #[derive::db_compatibility]
    pub async fn register(
        request: requests::Register,
        resp: oneshot::Sender<Result<(RegisterResponse, ID), requests::Status>>,
        db_connection: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        use entities::user::ActiveModel as UserModel;
        // Generate snowflake id
        let id = ID(utils::GENERATOR.generate()?.into_i64().try_into()?);
        // Generate ocid by random
        let ocid = utils::generate_ocid(consts::OCID_LEN);
        let passwd = request.password;
        let passwd =
            utils::spawn_blocking_with_tracing(move || compute_password_hash(&passwd)).await?;
        let user = UserModel {
            id: ActiveValue::Set(id.into()),
            ocid: ActiveValue::Set(ocid),
            passwd: ActiveValue::Set(passwd),
            name: ActiveValue::Set(request.name),
            email: ActiveValue::Set(request.email),
            time: ActiveValue::Set(chrono::Utc::now().timestamp().try_into()?),
            resource_used: ActiveValue::Set(0),
            friends_num: ActiveValue::Set(0),
            friend_limit: ActiveValue::Set(shared_state::get_friends_number_limit().try_into()?),
        };
        match user.insert(db_connection).await {
            Ok(res) => {
                // Happy Path
                let response = RegisterResponse::success(res.ocid);
                resp.send(Ok((response, res.id.into()))).unwrap();
            }
            Err(e) => {
                if let DbErr::RecordNotInserted = e {
                    resp.send(Err(requests::Status::Dup)).unwrap();
                } else {
                    tracing::error!("Database error:{e}");
                    resp.send(Err(requests::Status::ServerError)).unwrap();
                }
            }
        }
        Ok(())
    }

    #[derive::db_compatibility]
    pub async fn unregister(
        id: ID,
        resp: oneshot::Sender<requests::Status>,
        db_connection: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        use entities::user::ActiveModel as UserModel;
        let user = UserModel {
            id: ActiveValue::Set(id.into()),
            ..Default::default()
        };
        match user.delete(db_connection).await {
            Ok(_) => resp.send(requests::Status::Success)?,
            Err(e) => {
                tracing::error!("Database error:{e}");
                resp.send(requests::Status::ServerError)?;
            }
        }
        Ok(())
    }

    #[derive::db_compatibility]
    pub async fn new_session(
        _id: ID,
        _resp: oneshot::Sender<Result<NewSessionResponse, requests::Status>>,
        _db_connection: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    #[derive::db_compatibility]
    pub async fn up_load(
        id: ID,
        sz: Bt,
        resp: oneshot::Sender<requests::Status>,
        db_connection: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        use entities::user;
        let user_info = match user::Entity::find_by_id(id).one(db_connection).await? {
            Some(user) => user,
            None => {
                resp.send(requests::Status::ServerError)?;
                return Ok(());
            }
        };
        // first check if the limit has been reached
        let limit = shared_state::get_user_files_store_limit();
        let bytes_num: Bt = limit.into();
        let res_used: u64 = user_info.resource_used.try_into()?;
        let will_used = Bt(res_used + *sz);
        if will_used >= bytes_num {
            // reach the limit,delete some files to preserve the limit
            // TODO:clean files
        }
        let updated_res_lim = user_info.resource_used + 1;
        let mut user_info: user::ActiveModel = user_info.into();
        user_info.resource_used = ActiveValue::Set(updated_res_lim);
        user_info.update(db_connection).await?;
        resp.send(requests::Status::Success)?;
        Ok(())
    }
}

fn compute_password_hash(password: &str) -> String {
    // TODO:move factors to config
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string();
    password_hash
}

fn verify_password_hash(password: &str, password_hash: &str) -> anyhow::Result<()> {
    let expected = PasswordHash::new(password_hash).context("Not PHC string")?;
    argon2::Argon2::default()
        .verify_password(password.as_bytes(), &expected)
        .context("wrong password")?;
    Ok(())
}
