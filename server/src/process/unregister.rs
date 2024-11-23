use crate::{
    component::EmailSender,
    consts::ID,
    entities::{friend, operations, prelude::*, session_relation, user, user_chat_msg},
    pb::register::{UnregisterRequest, UnregisterResponse},
    server::RpcServer,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tonic::{Response, Status};

use super::get_id_from_req;

#[derive(Debug, thiserror::Error)]
enum ErrorOfUnregister {
    #[error("database error:{0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error:{0}")]
    UnknownError(#[from] anyhow::Error),
}

/// Remove user from database
async fn remove_account(
    id: ID,
    db_connection: &DatabaseConnection,
) -> Result<(), ErrorOfUnregister> {
    let user = user::ActiveModel {
        id: ActiveValue::Set(id.into()),
        ..Default::default()
    };
    user.delete(db_connection).await?;
    Ok(())
}

/// Remove all session related to the user
async fn remove_session_record(
    id: ID,
    db_conn: &DatabaseConnection,
) -> Result<(), ErrorOfUnregister> {
    let id: u64 = id.into();
    SessionRelation::delete_many()
        .filter(session_relation::Column::UserId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

async fn remove_msgs_of_user(
    id: ID,
    db_conn: &DatabaseConnection,
) -> Result<(), ErrorOfUnregister> {
    let id: u64 = id.into();
    UserChatMsg::delete_many()
        .filter(user_chat_msg::Column::SenderId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

async fn remove_friend(id: ID, db_conn: &DatabaseConnection) -> Result<(), ErrorOfUnregister> {
    let id: u64 = id.into();
    Friend::delete_many()
        .filter(friend::Column::UserId.eq(id))
        .exec(db_conn)
        .await?;
    Friend::delete_many()
        .filter(friend::Column::FriendId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

async fn remove_operations(id: ID, db_conn: &DatabaseConnection) -> Result<(), ErrorOfUnregister> {
    let id: u64 = id.into();
    Operations::delete_many()
        .filter(operations::Column::UserId.eq(id))
        .exec(db_conn)
        .await?;
    Ok(())
}

async fn unregister_impl(
    server: &RpcServer<impl EmailSender>,
    request: tonic::Request<UnregisterRequest>,
) -> Result<UnregisterResponse, ErrorOfUnregister> {
    let db_conn = &server.db;
    let id = get_id_from_req(&request).unwrap();
    let batch = async {
        remove_session_record(id, &db_conn.db_pool).await?;
        remove_msgs_of_user(id, &db_conn.db_pool).await?;
        remove_friend(id, &db_conn.db_pool).await?;
        remove_operations(id, &db_conn.db_pool).await?;
        remove_account(id, &db_conn.db_pool).await?;
        Ok(())
    };
    match batch.await {
        Ok(_) => Ok(UnregisterResponse {}),
        Err(e) => Err(e),
    }
}

pub async fn unregister<T: EmailSender>(
    server: &RpcServer<T>,
    request: tonic::Request<UnregisterRequest>,
) -> Result<tonic::Response<UnregisterResponse>, tonic::Status> {
    match unregister_impl(server, request).await {
        Ok(d) => Ok(Response::new(d)),
        Err(e) => {
            tracing::error!("{}", e);
            Err(Status::internal("Server Error"))
        }
    }
}
