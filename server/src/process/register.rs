use super::UserInfo;
use crate::{
    DbPool,
    component::EmailSender,
    consts::{self, ID},
    entities::user,
    pb::register::{RegisterRequest, RegisterResponse},
    server::AuthServiceProvider,
    shared_state, utils,
};
use anyhow::Context;
use argon2::{
    Params, PasswordHasher,
    password_hash::{self, SaltString},
};
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::num::TryFromIntError;
use tonic::{Request, Response, Status};
use tracing::error;

use super::generate_access_token;

async fn add_new_user(
    request: RegisterRequest,
    db_connection: &DbPool,
    params: Params,
) -> Result<(RegisterResponse, UserInfo), RegisterError> {
    // Generate snowflake id
    let id = ID(utils::GENERATOR
        .generate()
        .context("failed to generate snowflake id")?
        .into_i64()
        .try_into()?);
    // Generate ocid by random
    let ocid = utils::generate_ocid(consts::OCID_LEN);
    let passwd = request.password;
    let passwd = utils::spawn_blocking_with_tracing(move || compute_password_hash(&passwd, params))
        .await
        .context("compute hash error")?;
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ocid: ActiveValue::Set(ocid),
        passwd: ActiveValue::Set(passwd),
        name: ActiveValue::Set(request.name),
        email: ActiveValue::Set(request.email),
        time: ActiveValue::Set(chrono::Utc::now().into()),
        resource_used: ActiveValue::Set(0),
        friends_num: ActiveValue::Set(0),
        friend_limit: ActiveValue::Set(shared_state::get_friends_number_limit().try_into()?),
        ..Default::default()
    };
    match user.insert(&db_connection.db_pool).await {
        Ok(res) => {
            // Happy Path
            let response = RegisterResponse {
                ocid: res.ocid.clone(),
                token: generate_access_token(id),
            };
            Ok((response, UserInfo {
                ocid: res.ocid,
                id: res.id.into(),
            }))
        }
        Err(e) => {
            if let DbErr::RecordNotInserted = e {
                Err(RegisterError::UserExists)
            } else {
                Err(e.into())
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum RegisterError {
    #[error("User exists")]
    UserExists,
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    UnknownError(#[from] anyhow::Error),
    #[error("from int error")]
    FromIntError(#[from] TryFromIntError),
}

fn compute_password_hash(password: &str, params: Params) -> String {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash =
        argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();
    password_hash
}

async fn register_impl(
    server: &AuthServiceProvider<impl EmailSender>,
    request: Request<RegisterRequest>,
) -> Result<RegisterResponse, RegisterError> {
    let password_hash = &server.shared_data.cfg.main_cfg.password_hash;
    match add_new_user(
        request.into_inner(),
        &server.db,
        Params::new(
            password_hash.m_cost,
            password_hash.t_cost,
            password_hash.p_cost,
            password_hash.output_len,
        )
        .unwrap(),
    )
    .await
    {
        Ok((response, user_info)) => Ok(response),
        Err(e) => Err(e),
    }
}

pub async fn register<T: EmailSender>(
    server: &AuthServiceProvider<T>,
    request: Request<RegisterRequest>,
) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
    match register_impl(server, request).await {
        Ok(ok_resp) => Ok(Response::new(ok_resp)),
        Err(RegisterError::DbError(e)) => {
            error!("{}", e);
            Err(Status::internal("Database error"))
        }
        Err(RegisterError::UserExists) => Err(Status::already_exists("User exists")),
        Err(RegisterError::UnknownError(e)) => {
            error!("{}", e);
            Err(Status::internal("Unknown error"))
        }
        Err(RegisterError::FromIntError(e)) => {
            error!("{}", e);
            Err(Status::internal("Unknown error"))
        }
    }
}
