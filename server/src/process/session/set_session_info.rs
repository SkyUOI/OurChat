use crate::{component::EmailSender, server::RpcServer};
use pb::ourchat::session::set_session_info::v1::{SetSessionInfoRequest, SetSessionInfoResponse};
use tonic::{Request, Response, Status};

pub async fn set_session_info(
    server: &RpcServer<impl EmailSender>,
    request: Request<SetSessionInfoRequest>,
) -> Result<Response<SetSessionInfoResponse>, Status> {
    // process::get_session_info(self, request).await
    todo!()
}
