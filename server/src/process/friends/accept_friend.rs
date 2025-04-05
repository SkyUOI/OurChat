use crate::db::messages::{MsgError, insert_msg_record};
use crate::db::session::SessionError;
use crate::process::error_msg::{PERMISSION_DENIED, not_found};
use crate::process::friends::mapped_add_friend_to_redis;
use crate::process::{Dest, transmit_msg};
use crate::{db, helper, process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::Context;
use base::consts::ID;
use deadpool_redis::redis::AsyncCommands;
use migration::m20241229_022701_add_role_for_session::PredefinedRoles;
use pb::service::ourchat::friends::accept_friend::v1::{
    AcceptFriendNotification, AcceptFriendRequest, AcceptFriendResponse, AcceptFriendResult,
};
use pb::service::ourchat::friends::add_friend::v1::AddFriendRequest;
use pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondMsgType;
use pb::time::to_google_timestamp;
use sea_orm::{ActiveModelTrait, ActiveValue, TransactionTrait};
use tonic::{Request, Response, Status};

pub async fn accept_friend(
    server: &RpcServer,
    id: ID,
    request: Request<AcceptFriendRequest>,
) -> Result<Response<AcceptFriendResponse>, Status> {
    match accept_friend_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AcceptFriendErr::Db(_) | AcceptFriendErr::Internal(_) | AcceptFriendErr::Redis(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptFriendErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AcceptFriendErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}

impl From<SessionError> for AcceptFriendErr {
    fn from(value: SessionError) -> Self {
        match value {
            SessionError::Db(db_err) => Self::Db(db_err),
            SessionError::SessionNotFound => {
                tracing::error!("creating session but session not found was reported");
                Self::Status(Status::not_found(not_found::SESSION))
            }
        }
    }
}

impl From<MsgError> for AcceptFriendErr {
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

async fn accept_friend_impl(
    server: &RpcServer,
    id: ID,
    request: Request<AcceptFriendRequest>,
) -> Result<AcceptFriendResponse, AcceptFriendErr> {
    let req = request.into_inner();
    let inviter_id: ID = req.friend_id.into();
    let mut redis_conn = server
        .db
        .redis_pool
        .get()
        .await
        .context("cannot get redis connection")?;
    let key = mapped_add_friend_to_redis(inviter_id, id);
    let exist: bool = redis_conn.exists(&key).await?;
    if !exist {
        Err(Status::not_found(not_found::FRIEND_INVITATION))?;
    }
    let add_friend_req: String = redis_conn.get_del(&key).await?;
    let add_friend_req: AddFriendRequest = serde_json::from_str(&add_friend_req).unwrap();
    let mut session_id = None;
    if req.status == AcceptFriendResult::Success as i32 {
        // create a session
        session_id = Some(helper::generate_session_id()?);
        db::session::create_session_db(session_id.unwrap(), 0, "".to_owned(), &server.db.db_pool)
            .await?;
        let transaction = server.db.db_pool.begin().await?;
        db::session::batch_join_in_session(
            session_id.unwrap(),
            &[id, inviter_id],
            Some(PredefinedRoles::Owner.into()),
            &transaction,
        )
        .await?;
        transaction.commit().await?;

        // create friend relation
        let transaction = server.db.db_pool.begin().await?;
        let model = entities::friend::ActiveModel {
            user_id: ActiveValue::Set(inviter_id.into()),
            friend_id: ActiveValue::Set(id.into()),
        };
        model.insert(&transaction).await?;
        let model = entities::friend::ActiveModel {
            user_id: ActiveValue::Set(id.into()),
            friend_id: ActiveValue::Set(inviter_id.into()),
        };
        model.insert(&transaction).await?;
        if let Some(display_name) = add_friend_req.display_name {
            let model = entities::user_contact_info::ActiveModel {
                user_id: ActiveValue::Set(inviter_id.into()),
                contact_user_id: ActiveValue::Set(id.into()),
                display_name: ActiveValue::Set(Some(display_name)),
            };
            model.insert(&transaction).await?;
        }
        transaction.commit().await?;
    }
    // transmit to both
    let conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get redis connection")?;
    let mut channel = conn
        .create_channel()
        .await
        .context("cannot create channel")?;
    let respond_msg = RespondMsgType::AcceptFriend(AcceptFriendNotification {
        inviter_id: inviter_id.into(),
        invitee_id: id.into(),
        leave_message: req.leave_message,
        status: req.status,
        session_id: session_id.map(|x| x.into()),
    });
    // TODO: is_encrypted
    let transaction = server.db.db_pool.begin().await?;
    let _msg_model = insert_msg_record(
        inviter_id,
        None,
        respond_msg.clone(),
        false,
        &transaction,
        false,
    )
    .await?;
    let msg_model =
        insert_msg_record(id, None, respond_msg.clone(), false, &transaction, false).await?;
    transaction.commit().await?;
    let fetch_response = FetchMsgsResponse {
        msg_id: msg_model.msg_id as u64,
        time: Some(to_google_timestamp(msg_model.time.into())),
        respond_msg_type: Some(respond_msg),
    };
    transmit_msg(
        fetch_response,
        Dest::User(inviter_id),
        &mut channel,
        &server.db.db_pool,
    )
    .await?;
    let ret = AcceptFriendResponse {
        session_id: session_id.map(|x| x.into()),
    };
    Ok(ret)
}
