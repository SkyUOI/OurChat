use crate::{
    component::EmailSender,
    connection::db::get_id,
    consts::ID,
    entities::{prelude::*, session_relation, user, user_chat_msg},
    pb::register::{UnregisterRequest, UnregisterResponse},
    server::RpcServer,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
enum ErrorOfUnregister {
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
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

pub async fn unregister<T: EmailSender>(
    server: &RpcServer<T>,
    request: tonic::Request<UnregisterRequest>,
) -> Result<tonic::Response<UnregisterResponse>, tonic::Status> {
    let db_conn = &server.db.db_pool;
    let id = match get_id(&request.into_inner().ocid, &server.db).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("cannot get id from ocid:{e}");
            return Err(tonic::Status::internal("cannot get id from ocid"));
        }
    };
    let batch = async {
        remove_session_record(id, db_conn).await?;
        remove_msgs_of_user(id, db_conn).await?;
        remove_account(id, db_conn).await?;
        Ok(())
    };
    match batch.await {
        Ok(_) => Ok(Response::new(UnregisterResponse {})),
        Err(ErrorOfUnregister::DbError(e)) => {
            tracing::error!("Database error:{e}");
            Err(Status::internal("database error"))
        }
        Err(ErrorOfUnregister::UnknownError(e)) => {
            tracing::error!("Unknown error:{e}");
            Err(Status::internal("Unknown error"))
        }
    }
}
