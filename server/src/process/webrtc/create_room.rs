use crate::{process::error_msg::SERVER_ERROR, server::RpcServer};
use base::consts::ID;
use pb::service::ourchat::webrtc::room::create_room::v1::{CreateRoomRequest, CreateRoomResponse};
use tonic::{Request, Response, Status};

pub async fn create_room(
    server: &RpcServer,
    id: ID,
    request: Request<CreateRoomRequest>,
) -> Result<Response<CreateRoomResponse>, Status> {
    match create_room_impl(server, id, request).await {
        Ok(res) => Ok(Response::new(res)),
        Err(e) => match e {
            CreateRoomErr::Db(_) | CreateRoomErr::Internal(_) => {
                tracing::error!("{}", e);
                Err(Status::internal(SERVER_ERROR))
            }
            CreateRoomErr::Status(status) => Err(status),
        },
    }
}

#[derive(thiserror::Error, Debug)]
enum CreateRoomErr {
    #[error("database error:{0:?}")]
    Db(#[from] sea_orm::DbErr),
    #[error("status error:{0:?}")]
    Status(#[from] Status),
    #[error("internal error:{0:?}")]
    Internal(#[from] anyhow::Error),
}

async fn create_room_impl(
    _server: &RpcServer,
    _id: ID,
    _request: Request<CreateRoomRequest>,
) -> Result<CreateRoomResponse, CreateRoomErr> {
    todo!()
}
