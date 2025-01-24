//! Ourchat Server

use crate::process;
use crate::process::db::get_id;
use crate::{SERVER_INFO, SharedData, ShutdownRev};
use base::database::DbPool;
use base::time::to_google_timestamp;
use pb::auth::authorize::v1::{AuthRequest, AuthResponse};
use pb::auth::email_verify::v1::{VerifyRequest, VerifyResponse};
use pb::auth::register::v1::{RegisterRequest, RegisterResponse};
use pb::auth::v1::auth_service_server::{self, AuthServiceServer};
use pb::basic::server::v1::{RunningStatus, VERSION_SPLIT};
use pb::basic::v1::basic_service_server::{BasicService, BasicServiceServer};
use pb::basic::v1::{
    GetIdRequest, GetIdResponse, GetServerInfoRequest, TimestampRequest, TimestampResponse,
};
use pb::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::ourchat::get_account_info::v1::{GetAccountInfoRequest, GetAccountInfoResponse};
use pb::ourchat::msg_delivery::recall::v1::{RecallMsgRequest, RecallMsgResponse};
use pb::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, SendMsgRequest, SendMsgResponse,
};
use pb::ourchat::session::accept_session::v1::{AcceptSessionRequest, AcceptSessionResponse};
use pb::ourchat::session::add_role::v1::{AddRoleRequest, AddRoleResponse};
use pb::ourchat::session::get_session_info::v1::{GetSessionInfoRequest, GetSessionInfoResponse};
use pb::ourchat::session::new_session::v1::{NewSessionRequest, NewSessionResponse};
use pb::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
use pb::ourchat::session::set_session_info::v1::{SetSessionInfoRequest, SetSessionInfoResponse};
use pb::ourchat::set_account_info::v1::{
    SetFriendInfoRequest, SetFriendInfoResponse, SetSelfInfoRequest, SetSelfInfoResponse,
};
use pb::ourchat::unregister::v1::{UnregisterRequest, UnregisterResponse};
use pb::ourchat::upload::v1::{UploadRequest, UploadResponse};
use pb::ourchat::v1::our_chat_service_server::{OurChatService, OurChatServiceServer};
use process::error_msg::not_found;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use tokio::select;
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug)]
pub struct RpcServer {
    pub db: DbPool,
    pub shared_data: Arc<SharedData>,
    pub addr: SocketAddr,
    pub rabbitmq: deadpool_lapin::Pool,
}

impl RpcServer {
    pub fn new(
        ip: impl Into<SocketAddr>,
        db: DbPool,
        shared_data: Arc<SharedData>,
        rabbitmq: deadpool_lapin::Pool,
    ) -> Self {
        Self {
            db,
            shared_data,
            addr: ip.into(),
            rabbitmq,
        }
    }

    pub async fn run(self, mut shutdown_rev: ShutdownRev) -> anyhow::Result<()> {
        info!("starting rpc server, connecting to {}", self.addr);
        let addr = self.addr;
        let basic_service = BasicServiceProvider {
            shared_data: self.shared_data.clone(),
            db: self.db.clone(),
        };
        let auth_service = AuthServiceProvider {
            shared_data: self.shared_data.clone(),
            db: self.db.clone(),
            rabbitmq: self.rabbitmq.clone(),
        };
        let shared_data = self.shared_data.clone();
        let shared_data1 = self.shared_data.clone();
        let shared_data2 = self.shared_data.clone();
        let main_svc = OurChatServiceServer::with_interceptor(self, move |mut req| {
            shared_data.convert_maintaining_into_grpc_status()?;
            Self::check_auth(&mut req)?;
            Ok(req)
        });
        let basic_svc = BasicServiceServer::with_interceptor(basic_service, move |req| {
            shared_data1.convert_maintaining_into_grpc_status()?;
            Ok(req)
        });
        let auth_svc = AuthServiceServer::with_interceptor(auth_service, move |req| {
            shared_data2.convert_maintaining_into_grpc_status()?;
            Ok(req)
        });
        select! {
            _ = shutdown_rev.wait_shutting_down() => {}
            _ = tonic::transport::Server::builder()
                .add_service(main_svc)
                .add_service(basic_svc)
                .add_service(auth_svc)
                .serve(addr) => {}
        }
        Ok(())
    }

    fn check_auth(req: &mut Request<()>) -> Result<(), Status> {
        // check token
        match req.metadata().get("token") {
            Some(token) => {
                if let Some(jwt) = process::check_token(token.to_str().unwrap()) {
                    req.metadata_mut()
                        .insert("id", jwt.id.to_string().parse().unwrap());
                    Ok(())
                } else {
                    Err(Status::unauthenticated("Invalid token"))
                }
            }
            None => Err(Status::unauthenticated("Missing token")),
        }
    }
}

pub type FetchMsgsStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<FetchMsgsResponse, Status>> + Send>>;
pub type DownloadStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<DownloadResponse, Status>> + Send>>;

#[tonic::async_trait]
impl OurChatService for RpcServer {
    #[tracing::instrument(skip(self))]
    async fn unregister(
        &self,
        request: Request<UnregisterRequest>,
    ) -> Result<Response<UnregisterResponse>, Status> {
        process::unregister::unregister(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_account_info(
        &self,
        request: Request<GetAccountInfoRequest>,
    ) -> Result<Response<GetAccountInfoResponse>, Status> {
        process::get_account_info::get_info(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_self_info(
        &self,
        request: Request<SetSelfInfoRequest>,
    ) -> Result<Response<SetSelfInfoResponse>, Status> {
        process::set_account_info(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_friend_info(
        &self,
        request: Request<SetFriendInfoRequest>,
    ) -> Result<Response<SetFriendInfoResponse>, Status> {
        process::set_friend_info(self, request).await
    }

    type FetchMsgsStream = FetchMsgsStream;

    #[tracing::instrument(skip(self))]
    async fn fetch_msgs(
        &self,
        request: Request<FetchMsgsRequest>,
    ) -> Result<Response<Self::FetchMsgsStream>, Status> {
        process::fetch_user_msg(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn send_msg(
        &self,
        request: Request<SendMsgRequest>,
    ) -> Result<Response<SendMsgResponse>, Status> {
        process::send_msg(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn upload(
        &self,
        request: Request<tonic::Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        process::upload(self, request).await
    }

    type DownloadStream = DownloadStream;

    #[tracing::instrument(skip(self))]
    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        process::download(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn accept_session(
        &self,
        request: Request<AcceptSessionRequest>,
    ) -> Result<Response<AcceptSessionResponse>, Status> {
        process::accept_session(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn new_session(
        &self,
        request: Request<NewSessionRequest>,
    ) -> Result<Response<NewSessionResponse>, Status> {
        process::new_session(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_session_info(
        &self,
        request: Request<GetSessionInfoRequest>,
    ) -> Result<Response<GetSessionInfoResponse>, Status> {
        process::get_session_info(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_session_info(
        &self,
        request: Request<SetSessionInfoRequest>,
    ) -> Result<Response<SetSessionInfoResponse>, Status> {
        process::set_session_info(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn recall_msg(
        &self,
        request: Request<RecallMsgRequest>,
    ) -> Result<Response<RecallMsgResponse>, Status> {
        process::recall_msg(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_role(
        &self,
        request: Request<SetRoleRequest>,
    ) -> Result<Response<SetRoleResponse>, Status> {
        process::set_role(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn add_role(
        &self,
        request: Request<AddRoleRequest>,
    ) -> Result<Response<AddRoleResponse>, Status> {
        process::add_role(self, request).await
    }
}

#[derive(Debug)]
pub struct AuthServiceProvider {
    pub shared_data: Arc<SharedData>,
    pub db: DbPool,
    pub rabbitmq: deadpool_lapin::Pool,
}

pub type VerifyStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<VerifyResponse, Status>> + Send>>;

#[tonic::async_trait]
impl auth_service_server::AuthService for AuthServiceProvider {
    #[tracing::instrument(skip(self))]
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        process::register::register(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn auth(&self, request: Request<AuthRequest>) -> Result<Response<AuthResponse>, Status> {
        process::auth::auth(self, request).await
    }

    type VerifyStream = VerifyStream;

    #[tracing::instrument(skip(self))]
    async fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> Result<Response<Self::VerifyStream>, Status> {
        process::verify::email_verify(self, request).await
    }
}

#[derive(Debug)]
struct BasicServiceProvider {
    pub shared_data: Arc<SharedData>,
    pub db: DbPool,
}

#[tonic::async_trait]
impl BasicService for BasicServiceProvider {
    #[tracing::instrument(skip(self))]
    async fn timestamp(
        &self,
        _request: Request<TimestampRequest>,
    ) -> Result<Response<TimestampResponse>, Status> {
        let time = chrono::Utc::now();
        let res = TimestampResponse {
            timestamp: Some(to_google_timestamp(time)),
        };
        Ok(Response::new(res))
    }

    #[tracing::instrument(skip(self))]
    async fn get_server_info(
        &self,
        _request: Request<GetServerInfoRequest>,
    ) -> Result<Response<pb::basic::server::v1::GetServerInfoResponse>, Status> {
        Ok(Response::new(
            pb::basic::server::v1::GetServerInfoResponse {
                http_port: self.shared_data.cfg.main_cfg.http_port.into(),
                status: self.shared_data.get_maintaining().into(),
                ..SERVER_INFO_RPC.clone()
            },
        ))
    }

    #[tracing::instrument(skip(self))]
    async fn get_id(
        &self,
        request: Request<GetIdRequest>,
    ) -> Result<Response<GetIdResponse>, Status> {
        let req = request.into_inner();
        match get_id(&req.ocid, &self.db).await {
            Ok(id) => Ok(Response::new(GetIdResponse { id: *id })),
            Err(_) => Err(Status::not_found(not_found::USER)),
        }
    }
}

static SERVER_INFO_RPC: LazyLock<pb::basic::server::v1::GetServerInfoResponse> =
    LazyLock::new(|| pb::basic::server::v1::GetServerInfoResponse {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0,
        status: RunningStatus::Normal as i32,
        unique_identifier: SERVER_INFO.unique_id.to_string(),
        server_name: SERVER_INFO.server_name.to_string(),
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_split() {
        let ver_concat = format!(
            "{}.{}.{}",
            VERSION_SPLIT.major, VERSION_SPLIT.minor, VERSION_SPLIT.patch
        );
        assert_eq!(ver_concat, base::build::PKG_VERSION);
    }
}
