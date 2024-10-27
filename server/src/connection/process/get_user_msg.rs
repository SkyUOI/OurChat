use crate::{
    DbPool,
    client::{MsgConvert, requests::GetUserMsgRequest, response::ErrorMsgResponse},
    connection::{NetSender, UserInfo},
    consts::ID,
    entities::{session_relation, user_chat_msg},
};
use base::time::TimeStamp;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityOrSelect, EntityTrait,
    JoinType, Paginator, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select,
    Statement,
};

pub async fn get_user_msg(
    user_info: &UserInfo,
    request: GetUserMsgRequest,
    net_sender: impl NetSender,
    db_pool: &DbPool,
) -> anyhow::Result<()> {
    let ret = match get_session_msgs(user_info.id, request.time, &db_pool.db_pool).await {
        Ok(_) => {
            todo!()
        }
        Err(e) => {
            tracing::error!("Database error:{e}");
            ErrorMsgResponse::new(
                crate::client::requests::Status::ServerError,
                "Database error",
            )
            .to_msg()
        }
    };
    net_sender.send(ret).await?;
    Ok(())
}

async fn get_session_msgs(
    user_id: ID,
    end_timestamp: TimeStamp,
    db_conn: &DatabaseConnection,
) -> anyhow::Result<()> {
    // TODO:move page_size to config file
    let user_id: u64 = user_id.into();

    let mut msgs = user_chat_msg::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT * FROM user_chat_msg
        WHERE time > $1 AND
        EXISTS (SELECT * FROM session_relation WHERE user_id = $2 AND session_id = user_chat_msg.session_id)"#,
                [],
            ))
            .paginate(db_conn, 1000);
    // let ret = move || async move {
    //     match msgs.fetch_and_next().await? {
    //         Some(v) => {
    //             let mut ret = vec![];
    //             for i in v {
    //                 ret.push(data_wrapper::Msg::from(i));
    //             }
    //             anyhow::Ok(Some(ret))
    //         }
    //         None => Ok(None),
    //     }
    // };
    Ok(())
}
