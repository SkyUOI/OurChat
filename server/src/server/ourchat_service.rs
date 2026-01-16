use std::pin::Pin;

use pb::service::ourchat::delete::v1::{DeleteFileRequest, DeleteFileResponse};
use pb::service::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::service::ourchat::session::e2eeize_and_dee2eeize_session::v1::{
    Dee2eeizeSessionRequest, Dee2eeizeSessionResponse, E2eeizeSessionRequest,
    E2eeizeSessionResponse,
};
use pb::service::ourchat::session::get_role::v1::{GetRoleRequest, GetRoleResponse};
use pb::service::ourchat::session::invite_user_to_session::v1::{
    InviteUserToSessionRequest, InviteUserToSessionResponse,
};
use pb::service::ourchat::session::session_room_key::v1::{
    SendRoomKeyRequest, SendRoomKeyResponse,
};
use pb::service::ourchat::webrtc::room::accept_room_invitation::v1::{
    AcceptRoomInvitationRequest, AcceptRoomInvitationResponse,
};
use pb::service::ourchat::webrtc::room::create_room::v1::{CreateRoomRequest, CreateRoomResponse};
use pb::service::ourchat::webrtc::room::demote_admin::v1::{
    DemoteRoomAdminRequest, DemoteRoomAdminResponse,
};
use pb::service::ourchat::webrtc::room::get_room_members::v1::{
    GetRoomMembersRequest, GetRoomMembersResponse,
};
use pb::service::ourchat::webrtc::room::invite_user::v1::{
    InviteUserToRoomRequest, InviteUserToRoomResponse,
};
use pb::service::ourchat::webrtc::room::join_room::v1::{JoinRoomRequest, JoinRoomResponse};
use pb::service::ourchat::webrtc::room::kick_user::v1::{
    KickUserFromRoomRequest, KickUserFromRoomResponse,
};
use pb::service::ourchat::webrtc::room::leave_room::v1::{LeaveRoomRequest, LeaveRoomResponse};
use pb::service::ourchat::webrtc::room::promote_admin::v1::{
    PromoteRoomAdminRequest, PromoteRoomAdminResponse,
};
use pb::service::ourchat::webrtc::signal::v1::{SignalRequest, SignalResponse};
use tonic::{Request, Response, Status};

use super::RpcServer;
use crate::process::{self, get_id_from_req};
use pb::service::ourchat::friends::accept_friend_invitation::v1::{
    AcceptFriendInvitationRequest, AcceptFriendInvitationResponse,
};
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
use pb::service::ourchat::session::accept_join_session_invitation::v1::{
    AcceptJoinSessionInvitationRequest, AcceptJoinSessionInvitationResponse,
};
use pb::service::ourchat::session::add_role::v1::{AddRoleRequest, AddRoleResponse};
use pb::service::ourchat::session::allow_user_join_session::v1::{
    AllowUserJoinSessionRequest, AllowUserJoinSessionResponse,
};
use pb::service::ourchat::session::ban::v1::{
    BanUserRequest, BanUserResponse, UnbanUserRequest, UnbanUserResponse,
};
use pb::service::ourchat::session::delete_session::v1::{
    DeleteSessionRequest, DeleteSessionResponse,
};
use pb::service::ourchat::session::get_session_info::v1::{
    GetSessionInfoRequest, GetSessionInfoResponse,
};
use pb::service::ourchat::session::join_session::v1::{JoinSessionRequest, JoinSessionResponse};
use pb::service::ourchat::session::kick::v1::{KickUserRequest, KickUserResponse};
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
    async fn delete_file(
        &self,
        request: Request<DeleteFileRequest>,
    ) -> Result<Response<DeleteFileResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::delete_file(self, id, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn accept_join_session_invitation(
        &self,
        request: Request<AcceptJoinSessionInvitationRequest>,
    ) -> Result<Response<AcceptJoinSessionInvitationResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_join_session_invitation(self, id, request).await
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
    async fn get_role(
        &self,
        request: Request<GetRoleRequest>,
    ) -> Result<Response<GetRoleResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::get_role(self, id, request).await
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
    async fn accept_friend_invitation(
        &self,
        request: Request<AcceptFriendInvitationRequest>,
    ) -> Result<Response<AcceptFriendInvitationResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_friend_invitation(self, id, request).await
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

    /// Kick a user from a session permanently
    #[tracing::instrument(skip(self))]
    async fn kick_user(
        &self,
        request: Request<KickUserRequest>,
    ) -> Result<Response<KickUserResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::kick_user(self, id, request).await
    }

    /// Accept a pending session join request
    #[tracing::instrument(skip(self))]
    async fn allow_user_join_session(
        &self,
        request: Request<AllowUserJoinSessionRequest>,
    ) -> Result<Response<AllowUserJoinSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::allow_user_join_session(self, id, request).await
    }

    async fn invite_user_to_session(
        &self,
        request: Request<InviteUserToSessionRequest>,
    ) -> Result<Response<InviteUserToSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::invite_user_to_session(self, id, request).await
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

    async fn send_room_key(
        &self,
        request: Request<SendRoomKeyRequest>,
    ) -> Result<Response<SendRoomKeyResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::send_room_key(self, id, request).await
    }

    async fn e2eeize_session(
        &self,
        request: Request<E2eeizeSessionRequest>,
    ) -> Result<Response<E2eeizeSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::e2eeize_session(self, id, request).await
    }

    async fn dee2eeize_session(
        &self,
        request: Request<Dee2eeizeSessionRequest>,
    ) -> Result<Response<Dee2eeizeSessionResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::dee2eeize_session(self, id, request).await
    }

    /// Join a WebRTC room for VoIP call
    #[tracing::instrument(skip(self))]
    async fn join_room(
        &self,
        request: Request<JoinRoomRequest>,
    ) -> Result<Response<JoinRoomResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::join_room(self, id, request).await
    }

    /// Leave a WebRTC room
    #[tracing::instrument(skip(self))]
    async fn leave_room(
        &self,
        request: Request<LeaveRoomRequest>,
    ) -> Result<Response<LeaveRoomResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::leave_room(self, id, request).await
    }

    /// Get members of a WebRTC room
    #[tracing::instrument(skip(self))]
    async fn get_room_members(
        &self,
        request: Request<GetRoomMembersRequest>,
    ) -> Result<Response<GetRoomMembersResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::get_room_members(self, id, request).await
    }

    /// Invite a user to a WebRTC room
    #[tracing::instrument(skip(self))]
    async fn invite_user_to_room(
        &self,
        request: Request<InviteUserToRoomRequest>,
    ) -> Result<Response<InviteUserToRoomResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::invite_user_to_room(self, id, request).await
    }

    /// Accept an invitation to a WebRTC room
    #[tracing::instrument(skip(self))]
    async fn accept_room_invitation(
        &self,
        request: Request<AcceptRoomInvitationRequest>,
    ) -> Result<Response<AcceptRoomInvitationResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::accept_room_invitation(self, id, request).await
    }

    /// Promote a room member to admin
    #[tracing::instrument(skip(self))]
    async fn promote_room_admin(
        &self,
        request: Request<PromoteRoomAdminRequest>,
    ) -> Result<Response<PromoteRoomAdminResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::promote_room_admin(self, id, request).await
    }

    /// Demote a room admin to member
    #[tracing::instrument(skip(self))]
    async fn demote_room_admin(
        &self,
        request: Request<DemoteRoomAdminRequest>,
    ) -> Result<Response<DemoteRoomAdminResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::demote_room_admin(self, id, request).await
    }

    /// Kick a user from a WebRTC room
    #[tracing::instrument(skip(self))]
    async fn kick_user_from_room(
        &self,
        request: Request<KickUserFromRoomRequest>,
    ) -> Result<Response<KickUserFromRoomResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::kick_user_from_room(self, id, request).await
    }

    /// Signal WebRTC peer connection (offer/answer/ICE)
    #[tracing::instrument(skip(self))]
    async fn signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<SignalResponse>, Status> {
        let id = get_id_from_req(&request).unwrap();
        self.check_account_status(id).await?;
        process::signal(self, id, request).await
    }
}
