//! OurChat Server

use crate::db::user::get_account_info_db;
use crate::process::basic::support::support;
use crate::process::db::get_id;
use crate::process::error_msg::{ACCOUNT_DELETED, SERVER_ERROR};
use crate::process::{self, get_id_from_req};
use crate::{SERVER_INFO, SharedData, ShutdownRev};
use base::consts::{ID, OCID, VERSION_SPLIT};
use base::database::DbPool;
use base::time::to_google_timestamp;
use migration::m20250301_005919_add_soft_delete_columns::AccountStatus;
use pb::service::auth::authorize::v1::{AuthRequest, AuthResponse};
use pb::service::auth::email_verify::v1::{VerifyRequest, VerifyResponse};
use pb::service::auth::register::v1::{RegisterRequest, RegisterResponse};
use pb::service::auth::v1::auth_service_server::{self, AuthServiceServer};
use pb::service::basic::server::v1::RunningStatus;
use pb::service::basic::support::v1::{SupportRequest, SupportResponse};
use pb::service::basic::v1::basic_service_server::{BasicService, BasicServiceServer};
use pb::service::basic::v1::{
    GetIdRequest, GetIdResponse, GetServerInfoRequest, TimestampRequest, TimestampResponse,
};
use pb::service::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::service::ourchat::friends::accept_friend::v1::{AcceptFriendRequest, AcceptFriendResponse};
use pb::service::ourchat::friends::add_friend::v1::{AddFriendRequest, AddFriendResponse};
use pb::service::ourchat::friends::delete_friend::v1::{DeleteFriendRequest, DeleteFriendResponse};
use pb::service::ourchat::friends::set_friend_info::v1::{
    SetFriendInfoRequest, SetFriendInfoResponse,
};
use pb::service::ourchat::get_account_info::v1::{GetAccountInfoRequest, GetAccountInfoResponse};
use pb::service::ourchat::msg_delivery::recall::v1::{RecallMsgRequest, RecallMsgResponse};
use pb::service::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, SendMsgRequest, SendMsgResponse,
};
use pb::service::ourchat::session::accept_session::v1::{
    AcceptSessionRequest, AcceptSessionResponse,
};
use pb::service::ourchat::session::add_role::v1::{AddRoleRequest, AddRoleResponse};
use pb::service::ourchat::session::ban::v1::{
    BanUserRequest, BanUserResponse, UnbanUserRequest, UnbanUserResponse,
};
use pb::service::ourchat::session::delete_session::v1::{
    DeleteSessionRequest, DeleteSessionResponse,
};
use pb::service::ourchat::session::get_session_info::v1::{
    GetSessionInfoRequest, GetSessionInfoResponse,
};
use pb::service::ourchat::session::join_in_session::v1::{
    AcceptJoinInSessionRequest, AcceptJoinInSessionResponse, JoinInSessionRequest,
    JoinInSessionResponse,
};
use pb::service::ourchat::session::leave_session::v1::{LeaveSessionRequest, LeaveSessionResponse};
use pb::service::ourchat::session::mute::v1::{
    MuteUserRequest, MuteUserResponse, UnmuteUserRequest, UnmuteUserResponse,
};
use pb::service::ourchat::session::new_session::v1::{NewSessionRequest, NewSessionResponse};
use pb::service::ourchat::session::set_role::v1::{SetRoleRequest, SetRoleResponse};
use pb::service::ourchat::session::set_session_info::v1::{
    SetSessionInfoRequest, SetSessionInfoResponse,
};
use pb::service::ourchat::set_account_info::v1::{SetSelfInfoRequest, SetSelfInfoResponse};
use pb::service::ourchat::unregister::v1::{UnregisterRequest, UnregisterResponse};
use pb::service::ourchat::upload::v1::{UploadRequest, UploadResponse};
use pb::service::ourchat::v1::our_chat_service_server::{OurChatService, OurChatServiceServer};
use pb::service::server_manage::delete_account::v1::{DeleteAccountRequest, DeleteAccountResponse};
use pb::service::server_manage::v1::server_manage_service_server::{
    ServerManageService, ServerManageServiceServer,
};
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

pub struct ServerManageServiceProvider {
    pub db: DbPool,
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
        let server_manage_service = ServerManageServiceProvider {
            db: self.db.clone(),
        };
        let shared_data = self.shared_data.clone();
        let shared_data1 = self.shared_data.clone();
        let shared_data2 = self.shared_data.clone();
        let shared_data3 = self.shared_data.clone();
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

        let server_manage_svc =
            ServerManageServiceServer::with_interceptor(server_manage_service, move |req| {
                shared_data3.convert_maintaining_into_grpc_status()?;
                Ok(req)
            });

        select! {
            _ = shutdown_rev.wait_shutting_down() => {}
            err = tonic::transport::Server::builder()
                .add_service(main_svc)
                .add_service(basic_svc)
                .add_service(auth_svc)
                .add_service(server_manage_svc)
                .serve(addr) => {
                    err?
                }
        }
        Ok(())
    }

    fn check_auth(req: &mut Request<()>) -> Result<ID, Status> {
        // check token
        match req.metadata().get("token") {
            Some(token) => {
                if let Some(jwt) = process::check_token(token.to_str().unwrap()) {
                    let ret = jwt.id;
                    req.metadata_mut()
                        .insert("id", jwt.id.to_string().parse().unwrap());
                    Ok(ret)
                } else {
                    Err(Status::unauthenticated("Invalid token"))
                }
            }
            None => Err(Status::unauthenticated("Missing token")),
        }
    }

    async fn check_account_status(&self, id: ID) -> Result<(), Status> {
        let account = match get_account_info_db(id, &self.db.db_pool)
            .await
            .map_err(|_| Status::internal(SERVER_ERROR))?
        {
            Some(account) => account,
            None => return Err(Status::unauthenticated(not_found::USER)),
        };

        if account.account_status == AccountStatus::Deleted as i32 {
            return Err(Status::unauthenticated(ACCOUNT_DELETED));
        }

        Ok(())
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
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::unregister::unregister(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_account_info(
        &self,
        request: Request<GetAccountInfoRequest>,
    ) -> Result<Response<GetAccountInfoResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::get_account_info::get_account_info(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_self_info(
        &self,
        request: Request<SetSelfInfoRequest>,
    ) -> Result<Response<SetSelfInfoResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::set_self_info(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_friend_info(
        &self,
        request: Request<SetFriendInfoRequest>,
    ) -> Result<Response<SetFriendInfoResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::set_friend_info(self, id, request).await
    }

    type FetchMsgsStream = FetchMsgsStream;

    #[tracing::instrument(skip(self))]
    async fn fetch_msgs(
        &self,
        request: Request<FetchMsgsRequest>,
    ) -> Result<Response<Self::FetchMsgsStream>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::fetch_user_msg(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn send_msg(
        &self,
        request: Request<SendMsgRequest>,
    ) -> Result<Response<SendMsgResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::send_msg(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn upload(
        &self,
        request: Request<tonic::Streaming<UploadRequest>>,
    ) -> Result<Response<UploadResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::upload(self, id, request).await
    }

    type DownloadStream = DownloadStream;

    #[tracing::instrument(skip(self))]
    async fn download(
        &self,
        request: Request<DownloadRequest>,
    ) -> Result<Response<Self::DownloadStream>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::download(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn accept_session(
        &self,
        request: Request<AcceptSessionRequest>,
    ) -> Result<Response<AcceptSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_session(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn new_session(
        &self,
        request: Request<NewSessionRequest>,
    ) -> Result<Response<NewSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::new_session(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_session_info(
        &self,
        request: Request<GetSessionInfoRequest>,
    ) -> Result<Response<GetSessionInfoResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::get_session_info(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_session_info(
        &self,
        request: Request<SetSessionInfoRequest>,
    ) -> Result<Response<SetSessionInfoResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::set_session_info(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::delete_session(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn leave_session(
        &self,
        request: Request<LeaveSessionRequest>,
    ) -> Result<Response<LeaveSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::leave_session(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn recall_msg(
        &self,
        request: Request<RecallMsgRequest>,
    ) -> Result<Response<RecallMsgResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::recall_msg(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_role(
        &self,
        request: Request<SetRoleRequest>,
    ) -> Result<Response<SetRoleResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::set_role(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn add_role(
        &self,
        request: Request<AddRoleRequest>,
    ) -> Result<Response<AddRoleResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::add_role(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn mute_user(
        &self,
        request: Request<MuteUserRequest>,
    ) -> Result<Response<MuteUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::mute_user(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn unmute_user(
        &self,
        request: Request<UnmuteUserRequest>,
    ) -> Result<Response<UnmuteUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::unmute_user(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn ban_user(
        &self,
        request: Request<BanUserRequest>,
    ) -> Result<Response<BanUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::ban_user(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn unban_user(
        &self,
        request: Request<UnbanUserRequest>,
    ) -> Result<Response<UnbanUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::unban_user(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn add_friend(
        &self,
        request: Request<AddFriendRequest>,
    ) -> Result<Response<AddFriendResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::add_friend(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn accept_friend(
        &self,
        request: Request<AcceptFriendRequest>,
    ) -> Result<Response<AcceptFriendResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_friend(self, id, request).await
    }

    async fn delete_friend(
        &self,
        request: Request<DeleteFriendRequest>,
    ) -> Result<Response<DeleteFriendResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::delete_friend(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn join_in_session(
        &self,
        request: Request<JoinInSessionRequest>,
    ) -> Result<Response<JoinInSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::join_in_session(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn accept_join_in_session(
        &self,
        request: Request<AcceptJoinInSessionRequest>,
    ) -> Result<Response<AcceptJoinInSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_join_in_session(self, id, request).await
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
pub struct BasicServiceProvider {
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
    ) -> Result<Response<pb::service::basic::server::v1::GetServerInfoResponse>, Status> {
        Ok(Response::new(
            pb::service::basic::server::v1::GetServerInfoResponse {
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
        match get_id(&OCID(req.ocid), &self.db).await {
            Ok(id) => Ok(Response::new(GetIdResponse { id: *id })),
            Err(_) => Err(Status::not_found(not_found::USER)),
        }
    }

    async fn support(
        &self,
        request: Request<SupportRequest>,
    ) -> Result<Response<SupportResponse>, Status> {
        support(self, request).await
    }
}

static SERVER_INFO_RPC: LazyLock<pb::service::basic::server::v1::GetServerInfoResponse> =
    LazyLock::new(|| pb::service::basic::server::v1::GetServerInfoResponse {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0,
        status: RunningStatus::Normal as i32,
        unique_identifier: SERVER_INFO.unique_id.to_string(),
        server_name: SERVER_INFO.server_name.to_string(),
    });

#[tonic::async_trait]
impl ServerManageService for ServerManageServiceProvider {
    #[tracing::instrument(skip(self))]
    async fn delete_account(
        &self,
        request: Request<DeleteAccountRequest>,
    ) -> Result<Response<DeleteAccountResponse>, Status> {
        process::delete_account(self, request).await
    }
}

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
