use super::Server;
use crate::{
    connection::client_response::{self, LoginResponse, NewSessionResponse, RegisterResponse},
    consts::{self, ID},
    requests, utils,
};
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
    ) {
        use client_response::login::Status;
        use entities::prelude::*;
        use entities::user::Column;
        use requests::Status;
        // 判断帐号类型
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
                    if user.passwd == request.password {
                        match request.login_type {
                            requests::LoginType::Email => resp
                                .send(Ok((
                                    LoginResponse::success_email(user.ocid),
                                    user.id.try_into().unwrap(),
                                )))
                                .unwrap(),
                            requests::LoginType::Ocid => resp
                                .send(Ok((
                                    LoginResponse::success_ocid(),
                                    user.id.try_into().unwrap(),
                                )))
                                .unwrap(),
                        }
                    } else {
                        resp.send(Err(Status!(WrongPassword))).unwrap()
                    }
                }
                None => resp.send(Err(Status!(WrongPassword))).unwrap(),
            },
            Err(e) => {
                if let DbErr::RecordNotFound(_) = e {
                    resp.send(Err(Status!(WrongPassword))).unwrap()
                } else {
                    tracing::error!("database error:{}", e);
                    resp.send(Err(Status::ServerError)).unwrap()
                }
            }
        }
    }

    #[derive::db_compatibility]
    pub async fn register(
        request: requests::Register,
        resp: oneshot::Sender<Result<(RegisterResponse, ID), requests::Status>>,
        mysql_connection: &DatabaseConnection,
    ) {
        use entities::user::ActiveModel as UserModel;
        // 生成雪花id
        let id = utils::GENERATOR.generate().unwrap().into_i64() as ID;
        // 随机生成生成ocid
        let ocid = utils::generate_ocid(consts::OCID_LEN);
        let user = UserModel {
            id: sea_orm::ActiveValue::Set(id.try_into().unwrap()),
            ocid: sea_orm::ActiveValue::Set(ocid),
            passwd: sea_orm::ActiveValue::Set(request.password),
            name: sea_orm::ActiveValue::Set(request.name),
            email: sea_orm::ActiveValue::Set(request.email),
            time: sea_orm::ActiveValue::Set(chrono::Utc::now().timestamp().try_into().unwrap()),
        };
        match user.insert(mysql_connection).await {
            Ok(res) => {
                // 生成正确的响应
                let response = RegisterResponse::success(res.ocid);
                resp.send(Ok((response, res.id.try_into().unwrap())))
                    .unwrap();
            }
            Err(e) => {
                if let sea_orm::DbErr::RecordNotInserted = e {
                    resp.send(Err(requests::Status::Dup)).unwrap();
                } else {
                    tracing::error!("Database error:{e}");
                    resp.send(Err(requests::Status::ServerError)).unwrap();
                }
            }
        }
    }

    #[derive::db_compatibility]
    pub async fn unregister(
        id: ID,
        resp: oneshot::Sender<client_response::unregister::Status>,
        mysql_connection: &DatabaseConnection,
    ) {
        use entities::user::ActiveModel as UserModel;
        let user = UserModel {
            id: ActiveValue::Set(id.try_into().unwrap()),
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
        resp: oneshot::Sender<Result<NewSessionResponse, requests::Status>>,
        mysql_connection: &DatabaseConnection,
    ) {
    }
}
