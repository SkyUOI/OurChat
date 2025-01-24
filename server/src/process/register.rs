use super::generate_access_token;
use crate::process::error_msg::{SERVER_ERROR, exist};
use crate::{db, server::AuthServiceProvider, shared_state, utils};
use anyhow::Context;
use argon2::{Params, PasswordHasher, password_hash::SaltString};
use base::consts::{self, ID};
use base::database::DbPool;
use entities::user;
use pb::auth::register::v1::{RegisterRequest, RegisterResponse};
use sea_orm::sea_query::SubQueryOper::Exists;
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use snowdon::ClassicLayoutSnowflakeExtension;
use std::num::TryFromIntError;
use tonic::{Request, Response, Status};
use tracing::error;

async fn add_new_user(
    request: RegisterRequest,
    db_connection: &DbPool,
    params: Params,
) -> Result<RegisterResponse, RegisterError> {
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
    server: &AuthServiceProvider,
    request: Request<RegisterRequest>,
) -> Result<RegisterResponse, RegisterError> {
    let password_hash = &server.shared_data.cfg.main_cfg.password_hash;
    let response = add_new_user(
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
        },
    }
}
