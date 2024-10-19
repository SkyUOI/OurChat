use crate::{
    client::{MsgConvert, requests, response::RegisterResponse},
    connection::{NetSender, UserInfo, VerifyStatus},
    consts::{self, ID},
    shared_state, utils,
};
use argon2::{Params, PasswordHasher, password_hash::SaltString};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr};
use snowdon::ClassicLayoutSnowflakeExtension;

#[derive::db_compatibility]
async fn add_new_user(
    request: requests::RegisterRequest,
    db_connection: &DatabaseConnection,
) -> anyhow::Result<Result<(RegisterResponse, UserInfo), requests::Status>> {
    use entities::user::ActiveModel as UserModel;
    // Generate snowflake id
    let id = ID(utils::GENERATOR.generate()?.into_i64().try_into()?);
    // Generate ocid by random
    let ocid = utils::generate_ocid(consts::OCID_LEN);
    let passwd = request.password;
    let passwd = utils::spawn_blocking_with_tracing(move || compute_password_hash(&passwd)).await?;
    let user = UserModel {
        id: ActiveValue::Set(id.into()),
        ocid: ActiveValue::Set(ocid),
        passwd: ActiveValue::Set(passwd),
        name: ActiveValue::Set(request.name),
        email: ActiveValue::Set(request.email),
        time: ActiveValue::Set(chrono::Utc::now()),
        resource_used: ActiveValue::Set(0),
        friends_num: ActiveValue::Set(0),
        friend_limit: ActiveValue::Set(shared_state::get_friends_number_limit().try_into()?),
        ..Default::default()
    };
    match user.insert(db_connection).await {
        Ok(res) => {
            // Happy Path
            let response = RegisterResponse::success(res.ocid.clone());
            Ok(Ok((response, UserInfo {
                ocid: res.ocid,
                id: res.id.into(),
            })))
        }
        Err(e) => {
            if let DbErr::RecordNotInserted = e {
                Ok(Err(requests::Status::Dup))
            } else {
                tracing::error!("Database error:{e}");
                Ok(Err(requests::Status::ServerError))
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

/// Register Request
pub async fn register(
    net_sender: impl NetSender,
    register_data: requests::RegisterRequest,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<VerifyStatus> {
    match add_new_user(register_data, db_conn).await? {
        Ok(ok_resp) => {
            net_sender.send(ok_resp.0.to_msg()).await?;
            Ok(VerifyStatus::Success(ok_resp.1))
        }
        Err(e) => {
            net_sender
                .send(RegisterResponse::failed(e).to_msg())
                .await?;
            Ok(VerifyStatus::Fail)
        }
    }
}
