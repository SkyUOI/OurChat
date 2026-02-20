use crate::db::redis_mappings;
use crate::db::session::{get_members, if_permission_exist, leave_session};
use crate::process::error_msg::PERMISSION_DENIED;
use crate::process::error_msg::not_found::NOT_BE_BANNED;
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::constants::{ID, SessionID};
use migration::predefined::PredefinedPermissions;
use pb::service::ourchat::session::ban::v1::{
    BanUserRequest, BanUserResponse, UnbanUserRequest, UnbanUserResponse,
};
use redis::AsyncCommands;
use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

pub async fn ban_user(
    server: &RpcServer,
    id: ID,
    request: Request<BanUserRequest>,
) -> Result<Response<BanUserResponse>, Status> {
    match ban_user_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            BanUserErr::Db(_) | BanUserErr::Internal(_) | BanUserErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            BanUserErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum BanUserErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] redis::RedisError),
}

async fn ban_user_impl(
    server: &RpcServer,
    id: ID,
    request: Request<BanUserRequest>,
) -> Result<BanUserResponse, BanUserErr> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    if !if_permission_exist(
        id,
        req.session_id.into(),
        PredefinedPermissions::BanUser.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(BanUserErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    let mut conn = server.db.redis();
    let mut exec_ban_user = async |key| {
        match req.duration {
            Some(duration) => {
                let _: () = conn.set_ex(&key, "1", duration.seconds as u64).await?;
            }
            None => {
                let _: () = conn.set(&key, "1").await?;
            }
        }
        Result::<(), redis::RedisError>::Ok(())
    };
    let kick_user = async |kick_id| {
        tracing::info!("{} kicking...", kick_id);
        let transaction = server.db.db_pool.begin().await?;
        leave_session(session_id, kick_id, &transaction).await?;
        transaction.commit().await?;
        anyhow::Ok(())
    };
    for i in &req.user_ids {
        let user: ID = (*i).into();
        let key = redis_mappings::map_ban_to_redis(req.session_id.into(), user);
        exec_ban_user(key).await?;
        kick_user(user).await?;
    }
    // ban all
    if req.user_ids.is_empty() {
        let key = redis_mappings::map_ban_all_to_redis(req.session_id.into());
        exec_ban_user(key).await?;
        for i in get_members(session_id, &server.db.db_pool).await? {
            let user_id: ID = i.user_id.into();
            if !if_permission_exist(
                user_id,
                session_id,
                PredefinedPermissions::UnbanUser.into(),
                &server.db.db_pool,
            )
            .await?
            {
                kick_user(user_id).await?
            }
        }
    }
    Ok(BanUserResponse {})
}

pub async fn unban_user(
    server: &RpcServer,
    id: ID,
    request: Request<UnbanUserRequest>,
) -> Result<Response<UnbanUserResponse>, Status> {
    match unban_user_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            BanUserErr::Db(_) | BanUserErr::Internal(_) | BanUserErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            BanUserErr::Status(status) => Err(status),
        },
    }
}

async fn unban_user_impl(
    server: &RpcServer,
    id: ID,
    request: Request<UnbanUserRequest>,
) -> Result<UnbanUserResponse, BanUserErr> {
    let req = request.into_inner();
    if !if_permission_exist(
        id,
        req.session_id.into(),
        PredefinedPermissions::UnbanUser.into(),
        &server.db.db_pool,
    )
    .await?
    {
        return Err(BanUserErr::Status(Status::permission_denied(
            PERMISSION_DENIED,
        )));
    }
    let mut conn = server.db.redis();
    for i in req.user_ids {
        let user: ID = i.into();
        let key = redis_mappings::map_ban_to_redis(req.session_id.into(), user);

        // Check if key exists
        let exists: bool = conn.exists(&key).await?;
        if !exists {
            return Err(BanUserErr::Status(Status::not_found(NOT_BE_BANNED)));
        }
        let _: () = match conn.del(&key).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("{}", e);
                return Err(BanUserErr::Redis(e));
            }
        };
    }
    Ok(UnbanUserResponse {})
}
