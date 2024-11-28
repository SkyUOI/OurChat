use super::get_id_from_req;
use crate::{
    component::EmailSender,
    consts::ID,
    entities::user_chat_msg,
    pb::{
        self,
        ourchat::msg_delivery::v1::{FetchMsgRequest, Msg},
    },
    server::{FetchMsgStream, RpcServer},
    utils::from_google_timestamp,
};
use base::time::TimeStamp;
use sea_orm::{
    DatabaseBackend, DatabaseConnection, EntityTrait, Paginator, PaginatorTrait, Statement,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

#[derive(Debug, thiserror::Error)]
pub enum MsgError {
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),
}

pub async fn get_user_msg<T: EmailSender>(
    server: &RpcServer<T>,
    request: tonic::Request<FetchMsgRequest>,
) -> Result<Response<FetchMsgStream>, tonic::Status> {
    let id = get_id_from_req(&request).unwrap();
    let request = request.into_inner();
    let time = match from_google_timestamp(&match request.time {
        Some(t) => t,
        None => {
            return Err(tonic::Status::invalid_argument("time is missing"));
        }
    }) {
        Some(t) => t,
        None => {
            return Err(tonic::Status::invalid_argument("time error"));
        }
    };
    let (tx, rx) = mpsc::channel(32);
    let db_conn = server.db.clone();
    let fetch_page_size = server.shared_data.cfg.main_cfg.db.fetch_msg_page_size;
    tokio::spawn(async move {
        match get_session_msgs(id, time.into(), &db_conn.db_pool, fetch_page_size).await {
            Ok(mut pag) => {
                let db_logic = async {
                    while let Some(msgs) = pag.fetch_and_next().await? {
                        for msg in msgs {
                            let msg = match Msg::try_from(msg) {
                                Ok(m) => m,
                                Err(e) => {
                                    tracing::warn!("incorrect msg in database:{e}");
                                    continue;
                                }
                            };
                            match tx.send(Ok(msg)).await {
                                Ok(_) => {}
                                Err(e) => {
                                    tracing::error!("send msg error:{e}");
                                }
                            }
                        }
                    }
                    Ok::<(), MsgError>(())
                };
                match db_logic.await {
                    Ok(_) => {}
                    Err(MsgError::DbError(e)) => {
                        tracing::error!("Database error:{e}");
                        tx.send(Err(Status::internal("Database error"))).await.ok();
                    }
                    Err(MsgError::UnknownError(e)) => {
                        tracing::error!("Unknown error:{e}");
                        tx.send(Err(Status::internal("Unknown error"))).await.ok();
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error:{e}");
                tx.send(Err(Status::internal("Unknown error"))).await.ok();
            }
        }
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as FetchMsgStream))
}

async fn get_session_msgs(
    user_id: ID,
    end_timestamp: TimeStamp,
    db_conn: &DatabaseConnection,
    page_size: u64,
) -> Result<Paginator<'_, DatabaseConnection, sea_orm::SelectModel<user_chat_msg::Model>>, MsgError>
{
    let user_id: u64 = user_id.into();
    let msgs = user_chat_msg::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT * FROM user_chat_msg
        WHERE time > $1 AND
        EXISTS (SELECT * FROM session_relation WHERE user_id = $2 AND session_id = user_chat_msg.session_id)"#,
                [end_timestamp.into(), user_id.into()],
            ))
            .paginate(db_conn, page_size );
    Ok(msgs)
}
