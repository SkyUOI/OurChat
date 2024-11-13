use crate::{
    component::EmailSender,
    consts::ID,
    entities::user_chat_msg,
    pb::{
        self,
        msg_delivery::{OneMsg, SendMsgResponse},
    },
    server::{FetchMsgStream, RpcServer},
    utils::from_google_timestamp,
};
use base::time::TimeStamp;
use sea_orm::{
    DatabaseBackend, DatabaseConnection, EntityTrait, Paginator, PaginatorTrait, Statement,
};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Response;

use super::get_id_from_req;

#[derive(Debug, thiserror::Error)]
pub enum ErrorOfMsg {
    #[error("database error")]
    DbError(#[from] sea_orm::DbErr),
    #[error("unknown error")]
    UnknownError(#[from] anyhow::Error),
}

pub async fn get_user_msg<T: EmailSender>(
    server: &RpcServer<T>,
    request: tonic::Request<pb::msg_delivery::FetchMsgRequest>,
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
    tokio::spawn(async move {
        match get_session_msgs(id, time.into(), &db_conn.db_pool).await {
            Ok(mut pag) => {
                let db_logic = async {
                    while let Some(msgs) = pag.fetch_and_next().await? {
                        let mut msgs_map = HashMap::new();
                        for msg in msgs {
                            msgs_map
                                .entry(msg.session_id)
                                .or_insert_with(Vec::new)
                                .push(OneMsg::from(msg));
                        }
                        for i in msgs_map {
                            let session_id = i.0;
                            // let msg = GetUserMsgResponse::new(session_id.into(), i.1);
                        }
                    }
                    Ok::<(), ErrorOfMsg>(())
                };
                match db_logic.await {
                    Ok(_) => {}
                    Err(ErrorOfMsg::DbError(e)) => {
                        tracing::error!("Database error:{e}");
                        // net_sender
                        //     .send(ErrorMsgResponse::server_error("Database error").to_msg())
                        //     .await?;
                    }
                    Err(ErrorOfMsg::UnknownError(e)) => {
                        tracing::error!("Unknown error:{e}");
                        // net_sender
                        //     .send(ErrorMsgResponse::server_error("Unknown error").to_msg())
                        //     .await?;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error:{e}");
                // net_sender
                //     .send(
                //         ErrorMsgResponse::new(
                //             crate::client::requests::Status::ServerError,
                //             "Database error",
                //         )
                //         .to_msg(),
                //     )
                //     .await?;
            }
        };
    });
    let output_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(output_stream) as FetchMsgStream))
}

async fn get_session_msgs(
    user_id: ID,
    end_timestamp: TimeStamp,
    db_conn: &DatabaseConnection,
) -> Result<Paginator<'_, DatabaseConnection, sea_orm::SelectModel<user_chat_msg::Model>>, ErrorOfMsg>
{
    // TODO:move page_size to config file
    let user_id: u64 = user_id.into();
    let msgs = user_chat_msg::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT * FROM user_chat_msg
        WHERE time > $1 AND
        EXISTS (SELECT * FROM session_relation WHERE user_id = $2 AND session_id = user_chat_msg.session_id)"#,
                [end_timestamp.into(), user_id.into()],
            ))
            .paginate(db_conn, 2000);
    Ok(msgs)
}
