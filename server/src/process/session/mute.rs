use crate::db::redis;
use crate::db::redis::map_mute_all_to_redis;
use crate::db::session::if_permission_exist;
use crate::process::error_msg::{PERMISSION_DENIED, SERVER_ERROR, not_found::NOT_BE_MUTED};
use crate::server::RpcServer;
use anyhow::Context;
use base::consts::ID;
use deadpool_redis::redis::AsyncCommands;
use migration::m20241229_022701_add_role_for_session::PredefinedPermissions;
use pb::service::ourchat::session::mute::v1::{
    MuteUserRequest, MuteUserResponse, UnmuteUserRequest, UnmuteUserResponse,
};
use tonic::{Request, Response, Status};

pub async fn mute_user(
    server: &RpcServer,
    id: ID,
    request: Request<MuteUserRequest>,
) -> Result<Response<MuteUserResponse>, Status> {
    match mute_user_impl(server, id, request).await {
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

async fn mute_user_impl(
    server: &RpcServer,
    id: ID,
    request: Request<MuteUserRequest>,
) -> Result<MuteUserResponse, MuteUserErr> {
    let req = request.into_inner();
    if !if_permission_exist(
        id,
        req.session_id.into(),
        PredefinedPermissions::MuteUser.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(MuteUserErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    let mut conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;
    let mut set_mute_in_redis = async |key| {
        match req.duration {
            Some(duration) => {
                let _: () = conn.set_ex(&key, "1", duration.seconds as u64).await?;
            }
            None => {
                let _: () = conn.set(&key, "1").await?;
            }
        }
        Result::<(), deadpool_redis::redis::RedisError>::Ok(())
    };

    for i in &req.user_ids {
        let key = redis::map_mute_to_redis(req.session_id.into(), (*i).into());
        set_mute_in_redis(key).await?;
    }
    // mute all users
    if req.user_ids.is_empty() {
        let key = map_mute_all_to_redis(req.session_id.into());
        set_mute_in_redis(key).await?;
    }
    Ok(MuteUserResponse {})
}

pub async fn unmute_user(
    server: &RpcServer,
    id: ID,
    request: Request<UnmuteUserRequest>,
) -> Result<Response<UnmuteUserResponse>, Status> {
    match unmute_user_impl(server, id, request).await {
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
    id: ID,
    request: Request<UnmuteUserRequest>,
) -> Result<UnmuteUserResponse, MuteUserErr> {
    let req = request.into_inner();
    if !if_permission_exist(
        id,
        req.session_id.into(),
        PredefinedPermissions::UnmuteUser.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(MuteUserErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    // remove it in redis
    let mut conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;

    for i in req.user_ids {
        let user: ID = i.into();
        let key = redis::map_mute_to_redis(req.session_id.into(), user);
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
