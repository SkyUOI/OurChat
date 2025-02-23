use crate::process::error_msg::{CANNOT_SET_AVATAR, CANNOT_SET_DESCRIPTION, CANNOT_SET_NAME};
use crate::{
    db,
    process::{
        error_msg::{CONFLICT, SERVER_ERROR},
        get_id_from_req,
    },
    server::RpcServer,
};
use base::consts::SessionID;
use entities::{role_permissions, user_role_relation};
use migration::m20241229_022701_add_role_for_session::PredefinedPermissions;
use pb::service::ourchat::session::set_session_info::v1::{
    SetSessionInfoRequest, SetSessionInfoResponse,
};
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
                    Status::internal(SERVER_ERROR)
                }
                SetSessionErr::Status(s) => s,
                SetSessionErr::Conflict => Status::already_exists(CONFLICT),
            };
            Err(status)
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum SetSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("conflict")]
    Conflict,
}

async fn set_session_info_impl(
    server: &RpcServer,
    request: Request<SetSessionInfoRequest>,
) -> Result<SetSessionInfoResponse, SetSessionErr> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    let res = SetSessionInfoResponse {};
    let session_id: SessionID = request.session_id.into();
    let mut model = entities::session::ActiveModel {
        session_id: ActiveValue::Set(session_id.into()),
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
        if !permissions_map.contains(&(PredefinedPermissions::SetTitle as i64)) {
            return Err(SetSessionErr::Status(Status::permission_denied(
                CANNOT_SET_NAME,
            )));
        }
        model.name = ActiveValue::Set(name);
    }
    if let Some(description) = request.description {
        if !permissions_map.contains(&(PredefinedPermissions::SetDescription as i64)) {
            return Err(SetSessionErr::Status(Status::permission_denied(
                CANNOT_SET_DESCRIPTION,
            )));
        }
        model.description = ActiveValue::Set(description);
    }
    if let Some(avatar_key) = request.avatar_key {
        if !permissions_map.contains(&(PredefinedPermissions::SetAvatar as i64)) {
            return Err(SetSessionErr::Status(Status::permission_denied(
                CANNOT_SET_AVATAR,
            )));
        }
        model.avatar_key = ActiveValue::Set(Some(avatar_key));
    }
    match model.update(&server.db.db_pool).await {
        Ok(_) => Ok(res),
        Err(e) => {
            if db::helper::is_conflict(&e) {
                return Err(SetSessionErr::Conflict);
            }
            Err(SetSessionErr::Db(e))
        }
    }
}
