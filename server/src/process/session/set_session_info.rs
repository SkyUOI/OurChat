use crate::{db, process::get_id_from_req, server::RpcServer};
use entities::{role_permissions, user_role_relation};
use migration::m20241229_022701_add_role_for_session::PreDefinedPermissions;
use pb::ourchat::session::set_session_info::v1::{SetSessionInfoRequest, SetSessionInfoResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashSet;
use tonic::{Request, Response, Status};

pub async fn set_session_info(
    server: &RpcServer,
    request: Request<SetSessionInfoRequest>,
) -> Result<Response<SetSessionInfoResponse>, Status> {
    match set_session_info_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            let status = match e {
                SetSessionErr::Db(_) | SetSessionErr::Internal(_) => {
                    tracing::error!("{}", e);
                    Status::internal("Server Error")
                }
                SetSessionErr::Status(s) => s,
                SetSessionErr::Conflic => Status::already_exists("Conflict"),
            };
            Err(status)
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum SetSessionErr {
    #[error("database error:{0}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0}")]
    Status(#[from] tonic::Status),
    #[error("internal error:{0}")]
    Internal(#[from] anyhow::Error),
    #[error("conflict")]
    Conflic,
}

async fn set_session_info_impl(
    server: &RpcServer,
    request: Request<SetSessionInfoRequest>,
) -> Result<SetSessionInfoResponse, SetSessionErr> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    let res = SetSessionInfoResponse {};
    let mut model = entities::session::ActiveModel {
        ..Default::default()
    };
    // check the privilege
    // get all roles first
    let roles = user_role_relation::Entity::find()
        .filter(user_role_relation::Column::UserId.eq(id))
        .all(&server.db.db_pool)
        .await?;
    let mut permissions_map = HashSet::new();
    for i in &roles {
        let permissions_queried = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.eq(i.role_id))
            .all(&server.db.db_pool)
            .await?;
        for j in permissions_queried {
            permissions_map.insert(j.permission_id);
        }
    }
    if let Some(name) = request.name {
        if !permissions_map.contains(&(PreDefinedPermissions::SetTitle as i64)) {
            return Err(SetSessionErr::Status(Status::permission_denied(
                "Cannot set name",
            )));
        }
        model.name = ActiveValue::Set(name);
    }
    if let Some(description) = request.description {
        if !permissions_map.contains(&(PreDefinedPermissions::SetDescription as i64)) {
            return Err(SetSessionErr::Status(Status::permission_denied(
                "Cannot set description",
            )));
        }
        model.description = ActiveValue::Set(description);
    }
    if let Some(avatar_key) = request.avatar_key {
        if !permissions_map.contains(&(PreDefinedPermissions::SetAvatar as i64)) {
            return Err(SetSessionErr::Status(Status::permission_denied(
                "Cannot set avatar",
            )));
        }
        model.avatar_key = ActiveValue::Set(Some(avatar_key));
    }
    match model.update(&server.db.db_pool).await {
        Ok(_) => Ok(res),
        Err(e) => {
            if db::helper::is_conflict(&e) {
                return Err(SetSessionErr::Conflic);
            }
            Err(SetSessionErr::Db(e))
        }
    }
}
