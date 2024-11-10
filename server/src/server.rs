//! Ourchat Server

pub mod httpserver;

use crate::component::EmailSender;
use crate::connection::process;
use crate::pb::service::our_chat_service_server::{OurChatService, OurChatServiceServer};
use crate::{DbPool, HttpSender, SharedData, ShutdownRev, pb};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::select;

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
        select! {
            _ = shutdown_rev.wait_shutdowning() => {

            }
            _ = tonic::transport::Server::builder().add_service(OurChatServiceServer::new(self)).serve(addr) => {}
        }
        Ok(())
    }
}

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
        todo!()
    }

    async fn set_friend_info(
        &self,
        request: tonic::Request<pb::set_info::SetFriendInfoRequest>,
    ) -> Result<tonic::Response<pb::set_info::SetAccountInfoResponse>, tonic::Status> {
        todo!()
    }
}
