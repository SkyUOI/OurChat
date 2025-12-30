use crate::db::session::{SessionError, get_session_by_id, join_in_session, user_banned_status};
use crate::db::user::get_account_info_db;
use crate::process::error_msg::not_found;
use crate::process::{Dest, MsgInsTransmitErr, error_msg, message_insert_and_transmit};
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use anyhow::{Context, anyhow};
use base::consts::{ID, SessionID};
use entities::session_invitation;
use pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType;
use pb::service::ourchat::session::accept_join_session_invitation::v1::{
    AcceptJoinSessionInvitationRequest, AcceptJoinSessionInvitationResponse,
};
use pb::service::ourchat::session::invite_user_to_session::v1::AcceptSessionNotification;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, TransactionTrait};
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
enum AcceptJoinSessionInvitationError {
    #[error("database error:{0:?}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0:?}")]
    UnknownError(#[from] anyhow::Error),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("redis error:{0:?}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
    #[error("message error:{0:?}")]
    MessageError(#[from] MsgInsTransmitErr),
}

async fn accept_join_session_invitation_impl(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<AcceptJoinSessionInvitationRequest>,
) -> Result<AcceptJoinSessionInvitationResponse, AcceptJoinSessionInvitationError> {
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    let inviter = req.inviter_id;
    // check if banned from the session
    if user_banned_status(
        id,
        session_id,
        &mut server
            .db
            .redis_pool
            .get()
            .await
            .context("cannot get redis connection")?,
    )
    .await?
    .is_some()
    {
        Err(Status::permission_denied(error_msg::BAN))?;
    }
    // check if the invitation is valid
    let time_limit = chrono::Utc::now()
        - chrono::Duration::from_std(server.shared_data.cfg().main_cfg.verification_expire_time)
            .unwrap();
    let model = session_invitation::Entity::find()
        .filter(session_invitation::Column::SessionId.eq(req.session_id))
        .filter(session_invitation::Column::Invitee.eq(id))
        .filter(session_invitation::Column::ExpireAt.gt(time_limit))
        .one(&server.db.db_pool)
        .await?;
    match model {
        None => Err(Status::not_found(not_found::SESSION_INVITATION))?,
        Some(model) => {
            let transaction = server.db.db_pool.begin().await?;
            if req.accepted {
                match join_in_session(session_id, id, None, &transaction).await {
                    Ok(_) => {}
                    Err(SessionError::Db(e)) => {
                        transaction.rollback().await?;
                        return Err(AcceptJoinSessionInvitationError::DbError(e));
                    }
                    Err(SessionError::SessionNotFound) => {
                        transaction.rollback().await?;
                        return Err(AcceptJoinSessionInvitationError::Status(Status::not_found(
                            not_found::SESSION,
                        )));
                    }
                }
            }
            model.delete(&transaction).await?;
            transaction.commit().await?;
        }
    }
    let rmq_conn = server
        .rabbitmq
        .get()
        .await
        .context("cannot get rabbitmq connection")?;
    let mut conn = rmq_conn
        .create_channel()
        .await
        .context("cannot create rabbitmq channel")?;
    let session = get_session_by_id(session_id, &server.db.db_pool)
        .await?
        .ok_or(anyhow!("cannot find session"))?;
    let is_encrypted = session.e2ee_on;
    let user = get_account_info_db(id, &server.db.db_pool)
        .await?
        .ok_or(anyhow!("cannot find user"))?;
    let msg = RespondEventType::AcceptSessionApproval(AcceptSessionNotification {
        session_id: session_id.into(),
        accepted: req.accepted,
        public_key: (is_encrypted && req.accepted).then_some(user.public_key.into()),
        invitee_id: id.into(),
    });
    message_insert_and_transmit(
        id.into(),
        Some(session_id),
        msg,
        Dest::User(inviter.into()),
        false,
        &server.db.db_pool,
        &mut conn,
    )
    .await?;
    Ok(AcceptJoinSessionInvitationResponse {})
}

pub async fn accept_join_session_invitation(
    server: &RpcServer,
    id: ID,
    request: tonic::Request<AcceptJoinSessionInvitationRequest>,
) -> Result<Response<AcceptJoinSessionInvitationResponse>, Status> {
    match accept_join_session_invitation_impl(server, id, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => match e {
            AcceptJoinSessionInvitationError::DbError(_)
            | AcceptJoinSessionInvitationError::UnknownError(_)
            | AcceptJoinSessionInvitationError::Redis(_)
            | AcceptJoinSessionInvitationError::MessageError(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptJoinSessionInvitationError::Status(s) => Err(s),
        },
    }
}
