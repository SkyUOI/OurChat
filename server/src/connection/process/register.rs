use crate::{
    DbPool,
    component::EmailSender,
    connection::UserInfo,
    consts::{self, ID},
    entities::user,
    pb::register::{RegisterRequest, RegisterResponse},
    server::RpcServer,
    shared_state, utils,
};
use anyhow::anyhow;
use argon2::{Params, PasswordHasher, password_hash::SaltString};
use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use snowdon::ClassicLayoutSnowflakeExtension;
use tonic::{Request, Status};

use super::generate_access_token;

async fn add_new_user(
    request: RegisterRequest,
    db_connection: &DbPool,
) -> anyhow::Result<Result<(RegisterResponse, UserInfo), tonic::Status>> {
    // Generate snowflake id
    let id = ID(utils::GENERATOR.generate()?.into_i64().try_into()?);
    // Generate ocid by random
    let ocid = utils::generate_ocid(consts::OCID_LEN);
    let passwd = request.password;
    let passwd = utils::spawn_blocking_with_tracing(move || compute_password_hash(&passwd)).await?;
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
            Ok(Ok((response, UserInfo {
                ocid: res.ocid,
                id: res.id.into(),
            })))
        }
        Err(e) => {
            tracing::error!("Database error:{e}");
            if let DbErr::RecordNotInserted = e {
                Ok(Err(Status::already_exists("User already exists")))
            } else {
                Err(anyhow!(e))
            }
        }
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

pub async fn register<T: EmailSender>(
    server: &RpcServer<T>,
    request: Request<RegisterRequest>,
) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
    match add_new_user(request.into_inner(), &server.db).await {
        Ok(ok_resp) => match ok_resp {
            Ok((response, user_info)) => Ok(tonic::Response::new(response)),
            Err(e) => Err(e),
        },
        Err(e) => {
            tracing::error!("Database error:{e}");
            Err(tonic::Status::internal("database error"))
        }
    }
}
