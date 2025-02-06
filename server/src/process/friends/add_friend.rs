use crate::db::messages::{MsgError, insert_msg_record};
use crate::process::error_msg::exist::FRIEND;
use crate::process::error_msg::{PERMISSION_DENIED, not_found};
use crate::process::{Dest, friends, get_id_from_req, transmit_msg};
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::{ADD_FRIEND_REQUEST_EXPIRE, ID};
use base::time::to_google_timestamp;
use deadpool_redis::redis::AsyncCommands;
use entities::prelude::Friend;
use pb::service::ourchat::friends::add_friend::v1::{
    AddFriendApproval, AddFriendRequest, AddFriendResponse,
};
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use sea_orm::{EntityTrait, TransactionTrait};
use tonic::{Request, Response, Status};

pub async fn add_friend(
    server: &RpcServer,
    request: Request<AddFriendRequest>,
) -> Result<Response<AddFriendResponse>, Status> {
    match add_friend_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AddFriendErr::Db(_) | AddFriendErr::Internal(_) | AddFriendErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AddFriendErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AddFriendErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

impl From<MsgError> for AddFriendErr {
    fn from(value: MsgError) -> Self {
        match value {
            MsgError::DbError(db_err) => Self::Db(db_err),
            MsgError::UnknownError(error) => Self::Internal(error),
            MsgError::PermissionDenied => {
                Self::Status(Status::permission_denied(PERMISSION_DENIED))
            }
            MsgError::NotFound => {
                tracing::error!(
                    "Insert a new message record into the database, but a not found was returned."
                );
                Self::Status(Status::not_found(not_found::MSG))
            }
        }
    }
}

async fn add_friend_impl(
    server: &RpcServer,
    request: Request<AddFriendRequest>,
) -> Result<AddFriendResponse, AddFriendErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    let friend_id: ID = req.friend_id.into();
    let exist = Friend::find_by_id((id.into(), friend_id.into()))
        .one(&server.db.db_pool)
        .await?
        .is_some();
    if exist {
        Err(Status::already_exists(FRIEND))?;
    }
    // save invitation to redis
    let key = friends::mapped_add_friend_to_redis(id, friend_id);
    let mut conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;
    let _: () = conn
        .set_ex(
            &key,
            serde_json::to_string(&req).unwrap(),
            ADD_FRIEND_REQUEST_EXPIRE.as_secs(),
        )
        .await?;
    // insert 2 messages
    let respond_msg = RespondMsgType::AddFriendApproval(AddFriendApproval {
        inviter_id: id.into(),
        leave_message: req.leave_message,
    });
    // TODO: is_encrypted
    let transaction = server.db.db_pool.begin().await?;
    let _msg_model =
        insert_msg_record(friend_id, None, respond_msg.clone(), false, &transaction).await?;
    let msg_model = insert_msg_record(id, None, respond_msg.clone(), false, &transaction).await?;
    transaction.commit().await?;
    // send this message to the user who is invited
    let fetch_response = FetchMsgsResponse {
        msg_id: msg_model.chat_msg_id as u64,
        time: Some(to_google_timestamp(msg_model.time.into())),
        respond_msg_type: Some(respond_msg),
    };
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbitmq connection")?;
    let mut conn = rmq_conn
        .create_channel()
        .await
        .context("cannot create channel")?;
    transmit_msg(
        fetch_response,
        Dest::User(friend_id),
        &mut conn,
        &server.db.db_pool,
    )
    .await?;
    let ret = AddFriendResponse {};
    Ok(ret)
}
