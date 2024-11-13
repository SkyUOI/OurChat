//! Ourchat Server

pub mod httpserver;

use crate::component::EmailSender;
use crate::connection::process;
use crate::pb::service::basic_service_server::{BasicService, BasicServiceServer};
use crate::pb::service::our_chat_service_server::{OurChatService, OurChatServiceServer};
use crate::{DbPool, HttpSender, SharedData, ShutdownRev, pb, shared_state};
use prost_types::Timestamp;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use tokio::select;
use tonic::{Response, Status};

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
        let addr = self.addr;
        let basic_service = BasicServiceProvider {
            shared_data: self.shared_data.clone(),
        };
        select! {
            _ = shutdown_rev.wait_shutdowning() => {

            }
            _ = tonic::transport::Server::builder().add_service(OurChatServiceServer::new(self)).add_service(BasicServiceServer::new(basic_service)).serve(addr) => {}
        }
        Ok(())
    }
}

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
}

#[tonic::async_trait]
impl<T: EmailSender> BasicService for BasicServiceProvider<T> {
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
        major: ver[0].parse().unwrap(),
        minor: ver[1].parse().unwrap(),
        patch: ver[2].parse().unwrap(),
    }
});

static SERVER_INFO_RPC: LazyLock<pb::server::ServerInfo> =
    LazyLock::new(|| pb::server::ServerInfo {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0,
        status: pb::server::RunningStatus::Normal as i32,
    });
