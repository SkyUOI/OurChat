use crate::db::session::check_if_permission_exist;
use crate::process::error_msg::PERMISSION_DENIED;
use crate::process::error_msg::not_found::NOT_BE_MUTED;
use crate::process::get_id_from_req;
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::{ID, SessionID};
use deadpool_redis::redis::AsyncCommands;
use migration::m20241229_022701_add_role_for_session::PreDefinedPermissions;
use pb::ourchat::session::mute::v1::{
    MuteUserRequest, MuteUserResponse, UnmuteUserRequest, UnmuteUserResponse,
};
use tonic::{Request, Response, Status};

pub async fn mute_user(
    server: &RpcServer,
    request: Request<MuteUserRequest>,
) -> Result<Response<MuteUserResponse>, Status> {
    match mute_user_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            MuteUserErr::Db(_) | MuteUserErr::Internal(_) | MuteUserErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            MuteUserErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum MuteUserErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

fn map_mute_to_redis(session: SessionID, user_id: ID) -> String {
    format!("mute:{}:{}", session, user_id)
}

async fn mute_user_impl(
    server: &RpcServer,
    request: Request<MuteUserRequest>,
) -> Result<MuteUserResponse, MuteUserErr> {
    let id = get_id_from_req(&request).unwrap();
    if check_if_permission_exist(
        id,
        PreDefinedPermissions::MuteUser.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(MuteUserErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    let req = request.into_inner();
    for i in req.user_ids {
        let key = map_mute_to_redis(req.session_id.into(), i.into());

        let mut conn = server
            .db
            .redis_pool
            .get()
            .await
            .context("cannot get redis connection")?;
        match req.duration {
            Some(duration) => {
                let _: () = conn.set_ex(&key, "1", duration.seconds as u64).await?;
            }
            None => {
                let _: () = conn.set(&key, "1").await?;
            }
        }
    }
    Ok(MuteUserResponse {})
}

pub async fn unmute_user(
    server: &RpcServer,
    request: Request<UnmuteUserRequest>,
) -> Result<Response<UnmuteUserResponse>, Status> {
    match unmute_user_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            MuteUserErr::Db(_) | MuteUserErr::Internal(_) | MuteUserErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            MuteUserErr::Status(status) => Err(status),
        },
    }
}

async fn unmute_user_impl(
    server: &RpcServer,
    request: Request<UnmuteUserRequest>,
) -> Result<UnmuteUserResponse, MuteUserErr> {
    let id = get_id_from_req(&request).unwrap();
    if check_if_permission_exist(
        id,
        PreDefinedPermissions::UnmuteUser.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(MuteUserErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    // remove it in redis
    let req = request.into_inner();
    for i in req.user_ids {
        let user: ID = i.into();
        let key = map_mute_to_redis(req.session_id.into(), user);
        let mut conn = server
            .db
            .redis_pool
            .get()
            .await
            .context("cannot get redis connection")?;

        // Check if key exists
        let exists: bool = conn.exists(&key).await?;
        if !exists {
            return Err(MuteUserErr::Status(Status::not_found(NOT_BE_MUTED)));
        }

        let _: () = match conn.del(&key).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("{}", e);
                return Err(MuteUserErr::Redis(e));
            }
        };
    }
    Ok(UnmuteUserResponse {})
}
