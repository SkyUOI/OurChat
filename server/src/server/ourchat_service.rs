use std::pin::Pin;

use pb::service::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::service::ourchat::webrtc::room::create_room::v1::{CreateRoomRequest, CreateRoomResponse};
use tonic::{Request, Response, Status};

use super::RpcServer;
use crate::process::{self, get_id_from_req};
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
use pb::service::ourchat::session::join_session::v1::{
    AcceptJoinSessionRequest, AcceptJoinSessionResponse, JoinSessionRequest, JoinSessionResponse,
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
use pb::service::ourchat::v1::our_chat_service_server::OurChatService;

// Define stream types for streaming responses
pub type FetchMsgsStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<FetchMsgsResponse, Status>> + Send>>;
pub type DownloadStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<DownloadResponse, Status>> + Send>>;

/// Implementation of the main OurChat service
#[tonic::async_trait]
impl OurChatService for RpcServer {
    /// Unregister (delete) a user account
    /// Sets account status to deleted
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

    /// Mute a user in a session
    /// Prevents user from sending messages in the session
    #[tracing::instrument(skip(self))]
    async fn mute_user(
        &self,
        request: Request<MuteUserRequest>,
    ) -> Result<Response<MuteUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::mute_user(self, id, request).await
    }

    /// Unmute a previously muted user in a session
    /// Restores user's ability to send messages
    #[tracing::instrument(skip(self))]
    async fn unmute_user(
        &self,
        request: Request<UnmuteUserRequest>,
    ) -> Result<Response<UnmuteUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::unmute_user(self, id, request).await
    }

    /// Ban a user from a session
    /// Removes user from session and prevents rejoining
    #[tracing::instrument(skip(self))]
    async fn ban_user(
        &self,
        request: Request<BanUserRequest>,
    ) -> Result<Response<BanUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::ban_user(self, id, request).await
    }

    /// Unban a previous banned user from a session
    /// Allows user to rejoin the session
    #[tracing::instrument(skip(self))]
    async fn unban_user(
        &self,
        request: Request<UnbanUserRequest>,
    ) -> Result<Response<UnbanUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::unban_user(self, id, request).await
    }

    /// Send a friend request to another user
    #[tracing::instrument(skip(self))]
    async fn add_friend(
        &self,
        request: Request<AddFriendRequest>,
    ) -> Result<Response<AddFriendResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::add_friend(self, id, request).await
    }

    /// Accept a pending friend request
    #[tracing::instrument(skip(self))]
    async fn accept_friend(
        &self,
        request: Request<AcceptFriendRequest>,
    ) -> Result<Response<AcceptFriendResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_friend(self, id, request).await
    }

    /// Remove a user from the list of friends
    async fn delete_friend(
        &self,
        request: Request<DeleteFriendRequest>,
    ) -> Result<Response<DeleteFriendResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::delete_friend(self, id, request).await
    }

    /// Request to join a session
    #[tracing::instrument(skip(self))]
    async fn join_session(
        &self,
        request: Request<JoinSessionRequest>,
    ) -> Result<Response<JoinSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::join_session(self, id, request).await
    }

    /// Accept a pending session join request
    #[tracing::instrument(skip(self))]
    async fn accept_join_session(
        &self,
        request: Request<AcceptJoinSessionRequest>,
    ) -> Result<Response<AcceptJoinSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_join_session(self, id, request).await
    }

    /// Rpc create room
    #[tracing::instrument(skip(self))]
    async fn create_room(
        &self,
        request: Request<CreateRoomRequest>,
    ) -> Result<Response<CreateRoomResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::create_room(self, id, request).await
    }
}
