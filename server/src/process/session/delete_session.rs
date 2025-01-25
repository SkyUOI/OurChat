use crate::db::session::if_permission_exist;
use crate::process::get_id_from_req;
use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use migration::m20241229_022701_add_role_for_session::PreDefinedPermissions;
use pb::service::ourchat::session::delete_session::v1::{
    DeleteSessionRequest, DeleteSessionResponse,
};
use tonic::{Request, Response, Status};

pub async fn delete_session(
    server: &RpcServer,
    request: Request<DeleteSessionRequest>,
) -> Result<Response<DeleteSessionResponse>, Status> {
    match delete_session_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            DeleteSessionErr::Db(_) | DeleteSessionErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            DeleteSessionErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum DeleteSessionErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn delete_session_impl(
    server: &RpcServer,
    request: Request<DeleteSessionRequest>,
) -> Result<DeleteSessionResponse, DeleteSessionErr> {
    let id = get_id_from_req(&request).unwrap();
    let req = request.into_inner();
    // check permission
    if if_permission_exist(
        id,
        req.session_id.into(),
        PreDefinedPermissions::DeleteSession.into(),
        &server.db.db_pool,
    )
    .await?
    {}
    let ret = DeleteSessionResponse {};
    Ok(ret)
}
