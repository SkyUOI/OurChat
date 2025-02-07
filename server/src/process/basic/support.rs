use crate::server::BasicServiceProvider;
use pb::service::basic::support::v1::{SupportRequest, SupportResponse};
use tonic::{Request, Response, Status};

pub async fn support(
    server: &BasicServiceProvider,
    _request: Request<SupportRequest>,
) -> Result<Response<SupportResponse>, Status> {
    let ret = SupportResponse {};
    Ok(Response::new(ret))
}
