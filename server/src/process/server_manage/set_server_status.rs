use crate::process::error_msg::{self, SERVER_ERROR};
use crate::server::ServerManageServiceProvider;
use pb::service::server_manage::set_server_status::{
    self,
    v1::{SetServerStatusRequest, SetServerStatusResponse},
};
use tonic::{Request, Response, Status};

pub async fn set_server_status(
    server: &ServerManageServiceProvider,
    request: Request<SetServerStatusRequest>,
) -> Result<Response<SetServerStatusResponse>, Status> {
    match set_server_status_impl(server, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            SetServerStatusErr::Db(_) | SetServerStatusErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            SetServerStatusErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum SetServerStatusErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn set_server_status_impl(
    server: &ServerManageServiceProvider,
    request: Request<SetServerStatusRequest>,
) -> Result<SetServerStatusResponse, SetServerStatusErr> {
    let status = request.into_inner().server_status;
    match set_server_status::v1::ServerStatus::try_from(status) {
        Ok(status) => match status {
            set_server_status::v1::ServerStatus::Normal => {
                server.shared_data.set_maintaining(false);
            }
            set_server_status::v1::ServerStatus::Maintaining => {
                server.shared_data.set_maintaining(true);
            }
            set_server_status::v1::ServerStatus::Unspecified => {
                // ignore
            }
        },
        Err(_) => Err(Status::invalid_argument(error_msg::REQUEST_INVALID_VALUE))?,
    }
    Ok(SetServerStatusResponse {})
}
