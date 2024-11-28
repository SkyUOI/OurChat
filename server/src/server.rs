//! Ourchat Server

pub mod httpserver;

use crate::component::EmailSender;
use crate::pb::auth::authorize::v1::{AuthRequest, AuthResponse};
use crate::pb::auth::email_verify::v1::{VerifyRequest, VerifyResponse};
use crate::pb::auth::register::v1::{RegisterRequest, RegisterResponse};
use crate::pb::auth::v1::auth_service_server::{self, AuthServiceServer};
use crate::pb::basic::server::v1::{RunningStatus, ServerVersion};
use crate::pb::basic::v1::basic_service_server::{BasicService, BasicServiceServer};
use crate::pb::basic::v1::{GetServerInfoRequest, TimestampRequest, TimestampResponse};
use crate::pb::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use crate::pb::ourchat::get_account_info::v1::{GetAccountInfoRequest, GetAccountInfoResponse};
use crate::pb::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, Msg, SendMsgRequest, SendMsgResponse,
};
use crate::pb::ourchat::session::v1::{NewSessionRequest, NewSessionResponse};
use crate::pb::ourchat::set_account_info::v1::{
    SetAccountInfoResponse, SetFriendInfoRequest, SetSelfInfoRequest,
};
use crate::pb::ourchat::unregister::v1::{UnregisterRequest, UnregisterResponse};
use crate::pb::ourchat::upload::v1::{UploadRequest, UploadResponse};
use crate::pb::ourchat::v1::our_chat_service_server::{OurChatService, OurChatServiceServer};
use crate::utils::to_google_timestamp;
use crate::{DbPool, HttpSender, SharedData, ShutdownRev, pb, shared_state};
use crate::{ServerInfo, process};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use tokio::select;
use tonic::{Request, Response, Status};
use tracing::info;

pub struct RpcServer<T: EmailSender> {
    pub db: DbPool,
    pub http_sender: HttpSender,
    pub shared_data: Arc<SharedData<T>>,
    pub addr: SocketAddr,
}

impl<T: EmailSender> RpcServer<T> {
    pub fn new(
        ip: impl Into<SocketAddr>,
        db: DbPool,
        http_sender: HttpSender,
        shared_data: Arc<SharedData<T>>,
    ) -> Self {
        Self {
            db,
            http_sender,
            shared_data,
            addr: ip.into(),
        }
    }

    pub async fn run(self, mut shutdown_rev: ShutdownRev) -> anyhow::Result<()> {
        info!("starting rpc server, connecting to {}", self.addr);
        let addr = self.addr;
        let basic_service = BasicServiceProvider {
            shared_data: self.shared_data.clone(),
        };
        let auth_service = AuthServiceProvider {
            shared_data: self.shared_data.clone(),
            db: self.db.clone(),
        };
        let svc = OurChatServiceServer::with_interceptor(self, Self::check_auth);
        select! {
            _ = shutdown_rev.wait_shutdowning() => {}
            _ = tonic::transport::Server::builder()
                .add_service(svc)
                .add_service(BasicServiceServer::new(basic_service))
                .add_service(AuthServiceServer::new(auth_service))
                .serve(addr) => {}
        }
        Ok(())
    }

    fn check_auth(mut req: Request<()>) -> Result<Request<()>, Status> {
        // check token
        match req.metadata().get("token") {
            Some(token) => {
                if let Some(jwt) = process::check_token(token.to_str().unwrap()) {
                    req.metadata_mut()
                        .insert("id", jwt.id.to_string().parse().unwrap());
                    Ok(req)
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
pub type SendMsgStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<SendMsgResponse, Status>> + Send>>;
pub type DownloadStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<DownloadResponse, Status>> + Send>>;

#[tonic::async_trait]
impl<T: EmailSender> OurChatService for RpcServer<T> {
    async fn unregister(
        &self,
        request: tonic::Request<UnregisterRequest>,
    ) -> Result<tonic::Response<UnregisterResponse>, tonic::Status> {
        process::unregister::unregister(self, request).await
    }

    async fn get_info(
        &self,
        request: tonic::Request<GetAccountInfoRequest>,
    ) -> Result<tonic::Response<GetAccountInfoResponse>, tonic::Status> {
        process::get_account_info::get_info(self, request).await
    }

    async fn set_self_info(
        &self,
        request: tonic::Request<SetSelfInfoRequest>,
    ) -> Result<tonic::Response<SetAccountInfoResponse>, tonic::Status> {
        process::set_account_info(self, request).await
    }

    async fn set_friend_info(
        &self,
        request: tonic::Request<SetFriendInfoRequest>,
    ) -> Result<tonic::Response<SetAccountInfoResponse>, tonic::Status> {
        process::set_friend_info(self, request).await
    }

    type FetchMsgsStream = FetchMsgsStream;

    async fn fetch_msgs(
        &self,
        request: tonic::Request<FetchMsgsRequest>,
    ) -> Result<Response<Self::FetchMsgsStream>, tonic::Status> {
        process::get_user_msg(self, request).await
    }

    type SendMsgStream = SendMsgStream;

    async fn send_msg(
        &self,
        request: tonic::Request<tonic::Streaming<SendMsgRequest>>,
    ) -> Result<Response<Self::SendMsgStream>, tonic::Status> {
        process::send_msg(self, request).await
    }

    async fn upload(
        &self,
        request: tonic::Request<tonic::Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, tonic::Status> {
        process::upload(self, request).await
    }

    async fn new_session(
        &self,
        request: tonic::Request<NewSessionRequest>,
    ) -> Result<tonic::Response<NewSessionResponse>, tonic::Status> {
        process::new_session(self, request).await
    }

    type DownloadStream = DownloadStream;

    async fn download(
        &self,
        request: tonic::Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, tonic::Status> {
        process::download(self, request).await
    }
}

pub struct AuthServiceProvider<T: EmailSender> {
    pub shared_data: Arc<SharedData<T>>,
    pub db: DbPool,
}

pub type VerifyStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<VerifyResponse, Status>> + Send>>;

#[tonic::async_trait]
impl<T: EmailSender> auth_service_server::AuthService for AuthServiceProvider<T> {
    async fn auth(
        &self,
        request: tonic::Request<AuthRequest>,
    ) -> Result<tonic::Response<AuthResponse>, tonic::Status> {
        process::auth::auth(self, request).await
    }

    async fn register(
        &self,
        request: tonic::Request<RegisterRequest>,
    ) -> Result<tonic::Response<RegisterResponse>, tonic::Status> {
        process::register::register(self, request).await
    }

    type VerifyStream = VerifyStream;

    async fn verify(
        &self,
        request: tonic::Request<VerifyRequest>,
    ) -> Result<tonic::Response<Self::VerifyStream>, tonic::Status> {
        process::verify::email_verify(self, request).await
    }
}

struct BasicServiceProvider<T: EmailSender> {
    pub shared_data: Arc<SharedData<T>>,
}

#[tonic::async_trait]
impl<T: EmailSender> BasicService for BasicServiceProvider<T> {
    async fn timestamp(
        &self,
        _request: tonic::Request<TimestampRequest>,
    ) -> Result<tonic::Response<TimestampResponse>, tonic::Status> {
        let time = chrono::Utc::now();
        let res = TimestampResponse {
            timestamp: Some(to_google_timestamp(time)),
        };
        Ok(tonic::Response::new(res))
    }

    async fn get_server_info(
        &self,
        _request: tonic::Request<GetServerInfoRequest>,
    ) -> Result<tonic::Response<pb::basic::server::v1::GetServerInfoResponse>, tonic::Status> {
        Ok(Response::new(
            pb::basic::server::v1::GetServerInfoResponse {
                http_port: self.shared_data.cfg.main_cfg.http_port.into(),
                status: shared_state::get_maintaining().into(),
                ..*SERVER_INFO_RPC
            },
        ))
    }
}

static VERSION_SPLIT: LazyLock<ServerVersion> = LazyLock::new(|| {
    let ver = base::build::PKG_VERSION.split('.').collect::<Vec<_>>();
    ServerVersion {
        major: ver[0].parse().unwrap(),
        minor: ver[1].parse().unwrap(),
        patch: ver[2].parse().unwrap(),
    }
});

static SERVER_INFO_RPC: LazyLock<pb::basic::server::v1::GetServerInfoResponse> =
    LazyLock::new(|| pb::basic::server::v1::GetServerInfoResponse {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0,
        status: RunningStatus::Normal as i32,
    });
