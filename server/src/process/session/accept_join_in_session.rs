use crate::db::session::{SessionError, if_permission_exist};
use crate::process::error_msg::{PERMISSION_DENIED, not_found};
use crate::process::get_id_from_req;
use crate::{db, process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::SessionID;
use migration::m20241229_022701_add_role_for_session::PreDefinedPermissions;
use pb::service::ourchat::session::join_in_session::v1::{
    AcceptJoinInSessionRequest, AcceptJoinInSessionResponse,
};
use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

pub async fn accept_join_in_session(
    server: &RpcServer,
    request: Request<AcceptJoinInSessionRequest>,
) -> Result<Response<AcceptJoinInSessionResponse>, Status> {
    match accept_join_in_session_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            AcceptJoinInSessionErr::Db(_) | AcceptJoinInSessionErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            AcceptJoinInSessionErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum AcceptJoinInSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn accept_join_in_session_impl(
    server: &RpcServer,
    request: Request<AcceptJoinInSessionRequest>,
) -> Result<AcceptJoinInSessionResponse, AcceptJoinInSessionErr> {
    let id = get_id_from_req(&request).unwrap();
    // TODO: reply a failed response to the person
    let req = request.into_inner();
    let session_id: SessionID = req.session_id.into();
    if !if_permission_exist(
        id,
        session_id,
        PreDefinedPermissions::AcceptJoinRequest.into(),
        &server.db.db_pool,
    )
    .await?
    {
        Err(Status::permission_denied(PERMISSION_DENIED))?
    }
    if req.accepted {
        let transaction = server.db.db_pool.begin().await?;
        match db::session::join_in_session(session_id, req.user_id.into(), None, &transaction).await
        {
            Ok(_) => {
                transaction.commit().await?;
            }
            Err(e) => {
                transaction.rollback().await?;
                match e {
                    SessionError::Db(e) => {
                        Err(e)?;
                    }
                    SessionError::SessionNotFound => {
                        Err(Status::not_found(not_found::SESSION))?;
                    }
                }
            }
        }
    }
    let ret = AcceptJoinInSessionResponse {};
    Ok(ret)
}
