use super::Server;
use crate::{
    connection::client_response::{self, LoginResponse, NewSessionResponse, RegisterResponse},
    consts::{self, ID},
    entities, requests, utils,
};
use entities::user::ActiveModel as UserModel;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use snowdon::ClassicLayoutSnowflakeExtension;
use tokio::sync::oneshot;

impl Server {
    pub async fn login(
        request: requests::Login,
        resp: oneshot::Sender<Result<(LoginResponse, ID), client_response::login::Status>>,
        mysql_connection: &DatabaseConnection,
    ) {
        use client_response::login::Status;
        use entities::user::*;
        // 判断帐号类型
        let user = match request.login_type {
            requests::LoginType::Email => {
                Entity::find()
                    .filter(Column::Email.eq(request.account))
                    .one(mysql_connection)
                    .await
            }
            requests::LoginType::Ocid => {
                Entity::find()
                    .filter(Column::Ocid.eq(request.account))
                    .one(mysql_connection)
                    .await
            }
        };
        match user {
            Ok(data) => match data {
                Some(user) => {
                    if user.passwd == request.password {
                        match request.login_type {
                            requests::LoginType::Email => resp
                                .send(Ok((LoginResponse::success_email(user.ocid), user.id)))
                                .unwrap(),
                            requests::LoginType::Ocid => resp
                                .send(Ok((LoginResponse::success_ocid(), user.id)))
                                .unwrap(),
                        }
                    } else {
                        resp.send(Err(Status::WrongPassword)).unwrap()
                    }
                }
                None => resp.send(Err(Status::WrongPassword)).unwrap(),
            },
            Err(e) => {
                if let DbErr::RecordNotFound(_) = e {
                    resp.send(Err(Status::WrongPassword)).unwrap()
                } else {
                    tracing::error!("database error:{}", e);
                    resp.send(Err(Status::ServerError)).unwrap()
                }
            }
        }
    }

    pub async fn register(
        request: requests::Register,
        resp: oneshot::Sender<Result<(RegisterResponse, ID), client_response::register::Status>>,
        mysql_connection: &DatabaseConnection,
    ) {
        // 生成雪花id
        let id = utils::GENERATOR.generate().unwrap().into_i64() as ID;
        // 随机生成生成ocid
        let ocid = utils::generate_ocid(consts::OCID_LEN);
        let user = UserModel {
            id: sea_orm::ActiveValue::Set(id),
            ocid: sea_orm::ActiveValue::Set(ocid),
            passwd: sea_orm::ActiveValue::Set(request.password),
            name: sea_orm::ActiveValue::Set(request.name),
            email: sea_orm::ActiveValue::Set(request.email),
            time: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp() as u64),
        };
        match user.insert(mysql_connection).await {
            Ok(res) => {
                // 生成正确的响应
                let response = RegisterResponse::success(res.ocid);
                resp.send(Ok((response, res.id))).unwrap();
            }
            Err(e) => {
                if let sea_orm::DbErr::RecordNotInserted = e {
                    resp.send(Err(client_response::register::Status::Dup))
                        .unwrap();
                } else {
                    tracing::error!("Database error:{e}");
                    resp.send(Err(client_response::register::Status::ServerError))
                        .unwrap();
                }
            }
        }
    }

    pub async fn unregister(
        id: ID,
        resp: oneshot::Sender<client_response::unregister::Status>,
        mysql_connection: &DatabaseConnection,
    ) {
        let user = UserModel {
            id: ActiveValue::Set(id),
            ..Default::default()
        };
        use client_response::unregister::Status;
        match user.delete(mysql_connection).await {
            Ok(_) => resp.send(Status::Success).unwrap(),
            Err(e) => {
                tracing::error!("Database error:{e}");
                resp.send(Status::Failed).unwrap();
            }
        }
    }

    pub async fn new_session(
        id: ID,
        resp: oneshot::Sender<Result<NewSessionResponse, client_response::new_session::Status>>,
        mysql_connection: &DatabaseConnection,
    ) {
    }
}