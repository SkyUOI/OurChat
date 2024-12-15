//! Ourchat Server

pub mod httpserver;

use crate::component::EmailSender;
use crate::process;
use crate::process::db::get_id;
use crate::{DbPool, HttpSender, SERVER_INFO, SharedData, ShutdownRev, shared_state};
use base::time::to_google_timestamp;
use pb::auth::authorize::v1::{AuthRequest, AuthResponse};
use pb::auth::email_verify::v1::{VerifyRequest, VerifyResponse};
use pb::auth::register::v1::{RegisterRequest, RegisterResponse};
use pb::auth::v1::auth_service_server::{self, AuthServiceServer};
use pb::basic::server::v1::{RunningStatus, ServerVersion};
use pb::basic::v1::basic_service_server::{BasicService, BasicServiceServer};
use pb::basic::v1::{
    GetIdRequest, GetIdResponse, GetServerInfoRequest, TimestampRequest, TimestampResponse,
};
use pb::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::ourchat::get_account_info::v1::{GetAccountInfoRequest, GetAccountInfoResponse};
use pb::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, SendMsgRequest, SendMsgResponse,
};
use pb::ourchat::session::accept_session::v1::{AcceptSessionRequest, AcceptSessionResponse};
use pb::ourchat::session::get_session_info::v1::{GetSessionInfoRequest, GetSessionInfoResponse};
use pb::ourchat::session::new_session::v1::{NewSessionRequest, NewSessionResponse};
use pb::ourchat::set_account_info::v1::{
    SetFriendInfoRequest, SetFriendInfoResponse, SetSelfInfoRequest, SetSelfInfoResponse,
};
use pb::ourchat::unregister::v1::{UnregisterRequest, UnregisterResponse};
use pb::ourchat::upload::v1::{UploadRequest, UploadResponse};
use pb::ourchat::v1::our_chat_service_server::{OurChatService, OurChatServiceServer};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use tokio::select;
<<<<<<< HEAD
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug)]
=======
use tonic::{Response, Status};

>>>>>>> client-develop
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
<<<<<<< HEAD
        info!("starting rpc server, connecting to {}", self.addr);
        let addr = self.addr;
        let basic_service = BasicServiceProvider {
            shared_data: self.shared_data.clone(),
            db: self.db.clone(),
=======
        let addr = self.addr;
        let basic_service = BasicServiceProvider {
            shared_data: self.shared_data.clone(),
>>>>>>> client-develop
        };
        let auth_service = AuthServiceProvider {
            shared_data: self.shared_data.clone(),
            db: self.db.clone(),
        };
        let svc = OurChatServiceServer::with_interceptor(self, Self::check_auth);
        select! {
<<<<<<< HEAD
            _ = shutdown_rev.wait_shutdowning() => {}
            _ = tonic::transport::Server::builder()
                .add_service(svc)
                .add_service(BasicServiceServer::new(basic_service))
                .add_service(AuthServiceServer::new(auth_service))
                .serve(addr) => {}
=======
            _ = shutdown_rev.wait_shutdowning() => {

            }
            _ = tonic::transport::Server::builder().add_service(OurChatServiceServer::new(self)).add_service(BasicServiceServer::new(basic_service)).serve(addr) => {}
>>>>>>> client-develop
        }
        Ok(())
    }
}

<<<<<<< HEAD
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
pub type DownloadStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<DownloadResponse, Status>> + Send>>;

#[tonic::async_trait]
impl<T: EmailSender> OurChatService for RpcServer<T> {
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
        // process::get_session_info(self, request).await
        todo!()
    }
}

#[derive(Debug)]
pub struct AuthServiceProvider<T: EmailSender> {
    pub shared_data: Arc<SharedData<T>>,
    pub db: DbPool,
}

pub type VerifyStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<VerifyResponse, Status>> + Send>>;

#[tonic::async_trait]
impl<T: EmailSender> auth_service_server::AuthService for AuthServiceProvider<T> {
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
struct BasicServiceProvider<T: EmailSender> {
    pub shared_data: Arc<SharedData<T>>,
    pub db: DbPool,
=======
pub type FetchMsgStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<pb::msg_delivery::Msg, Status>> + Send>>;

#[tonic::async_trait]
impl<T: EmailSender> OurChatService for RpcServer<T> {
    async fn register(
        &self,
        request: tonic::Request<pb::register::RegisterRequest>,
    ) -> Result<tonic::Response<pb::register::RegisterResponse>, tonic::Status> {
        process::register::register(self, request).await
    }

    async fn unregister(
        &self,
        request: tonic::Request<pb::register::UnregisterRequest>,
    ) -> Result<tonic::Response<pb::register::UnregisterResponse>, tonic::Status> {
        process::unregister::unregister(self, request).await
    }

    async fn login(
        &self,
        request: tonic::Request<pb::login::LoginRequest>,
    ) -> Result<tonic::Response<pb::login::LoginResponse>, tonic::Status> {
        process::login::login(self, request).await
    }

    async fn get_info(
        &self,
        request: tonic::Request<pb::get_info::GetAccountInfoRequest>,
    ) -> Result<tonic::Response<pb::get_info::GetAccountInfoResponse>, tonic::Status> {
        process::get_account_info::get_info(self, request).await
    }

    async fn set_self_info(
        &self,
        request: tonic::Request<pb::set_info::SetSelfInfoRequest>,
    ) -> Result<tonic::Response<pb::set_info::SetAccountInfoResponse>, tonic::Status> {
        process::set_account_info(self, request).await
    }

    async fn set_friend_info(
        &self,
        request: tonic::Request<pb::set_info::SetFriendInfoRequest>,
    ) -> Result<tonic::Response<pb::set_info::SetAccountInfoResponse>, tonic::Status> {
        process::set_friend_info(self, request).await
    }

    type fetch_msgsStream = FetchMsgStream;

    async fn fetch_msgs(
        &self,
        request: tonic::Request<pb::msg_delivery::FetchMsgRequest>,
    ) -> Result<Response<Self::fetch_msgsStream>, tonic::Status> {
        todo!()
    }

    async fn msg_delivery(
        &self,
        request: tonic::Request<tonic::Streaming<pb::msg_delivery::SendMsgRequest>>,
    ) -> Result<Response<pb::msg_delivery::SendMsgResponse>, tonic::Status> {
        todo!()
    }
}

struct BasicServiceProvider<T: EmailSender> {
    pub shared_data: Arc<SharedData<T>>,
>>>>>>> client-develop
}

#[tonic::async_trait]
impl<T: EmailSender> BasicService for BasicServiceProvider<T> {
<<<<<<< HEAD
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
                status: shared_state::get_maintaining().into(),
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
            Err(e) => Err(Status::not_found("user not found")),
        }
    }
}

static VERSION_SPLIT: LazyLock<ServerVersion> = LazyLock::new(|| {
    let ver = base::build::PKG_VERSION.split('.').collect::<Vec<_>>();
    ServerVersion {
=======
    async fn timestamp(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<prost_types::Timestamp>, tonic::Status> {
        let time = chrono::Utc::now();
        let timestamp = Timestamp {
            seconds: time.timestamp(),
            nanos: time.timestamp_subsec_nanos() as i32,
        };
        Ok(tonic::Response::new(timestamp))
    }

    async fn get_server_info(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::server::ServerInfo>, tonic::Status> {
        Ok(Response::new(pb::server::ServerInfo {
            http_port: self.shared_data.cfg.main_cfg.http_port.into(),
            status: shared_state::get_maintaining().into(),
            ..*SERVER_INFO_RPC
        }))
    }
}

static VERSION_SPLIT: LazyLock<pb::server::ServerVersion> = LazyLock::new(|| {
    let ver = base::build::PKG_VERSION.split('.').collect::<Vec<_>>();
    pb::server::ServerVersion {
>>>>>>> client-develop
        major: ver[0].parse().unwrap(),
        minor: ver[1].parse().unwrap(),
        patch: ver[2].parse().unwrap(),
    }
});

<<<<<<< HEAD
static SERVER_INFO_RPC: LazyLock<pb::basic::server::v1::GetServerInfoResponse> =
    LazyLock::new(|| pb::basic::server::v1::GetServerInfoResponse {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0,
        status: RunningStatus::Normal as i32,
        unique_identifier: SERVER_INFO.unique_id.to_string(),
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
=======
static SERVER_INFO_RPC: LazyLock<pb::server::ServerInfo> =
    LazyLock::new(|| pb::server::ServerInfo {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0,
        status: pb::server::RunningStatus::Normal as i32,
    });
>>>>>>> client-develop
