use crate::ClientCore;
use crate::oc_helper::FAKE_MANAGER;
use crate::oc_helper::client::{OCClient, ServerManageClient};
use crate::oc_helper::{ClientErr, Clients};
use anyhow::Context;
use base::constants::{ID, JWT_HEADER, OCID, SessionID};
use base::setting::tls::TlsConfig;
use bytes::Bytes;
use migration::predefined::PredefinedServerManagementRole;
use pb::service::auth::authorize::v1::{AuthRequest, auth_request};
use pb::service::auth::register::v1::RegisterRequest;
use pb::service::basic::v1::TimestampRequest;
use pb::service::basic::v1::basic_service_client::BasicServiceClient;
use pb::service::ourchat::delete::v1::DeleteFileRequest;
use pb::service::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::service::ourchat::get_account_info;
use pb::service::ourchat::get_account_info::v1::{GetAccountInfoRequest, GetAccountInfoResponse};
use pb::service::ourchat::msg_delivery::v1::{
    FetchMsgsRequest, FetchMsgsResponse, SendMsgRequest, SendMsgResponse,
};
use pb::service::ourchat::session::accept_join_session_invitation::v1::AcceptJoinSessionInvitationRequest;
use pb::service::ourchat::session::ban::v1::{BanUserRequest, UnbanUserRequest};
use pb::service::ourchat::session::kick::v1::KickUserRequest;
use pb::service::ourchat::session::mute::v1::{MuteUserRequest, UnmuteUserRequest};
use pb::service::ourchat::unregister::v1::UnregisterRequest;
use pb::service::ourchat::upload::v1::{
    CancelUploadRequest, CompleteUploadRequest, StartUploadRequest, UploadChunkRequest,
};
use pb::service::ourchat::v1::our_chat_service_client::OurChatServiceClient;
use pb::service::server_manage::user_manage::v1::RemoveServerRoleRequest;
use pb::service::server_manage::v1::server_manage_service_client::ServerManageServiceClient;
use pb::time::TimeStampUtc;
use rand::RngExt;
use rsa::pkcs1::EncodeRsaPublicKey as _;
use rsa::{RsaPrivateKey, RsaPublicKey};
use server::helper::generate_random_string;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_stream::StreamExt;
use tonic::metadata::MetadataValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Uri};
use tonic::{Response, Status, Streaming};

pub struct TestUser {
    pub name: String,
    pub password: String,
    pub email: String,
    pub ocid: OCID,
    pub id: ID,
    pub port: u16,
    pub token: String,
    pub clients: Clients,
    pub rpc_url: String,
    pub oc_server: Option<OCClient>,
    pub server_manage_client: Option<ServerManageClient>,
    pub timestamp_receive_msg: TimeStampUtc,
    pub tls: TlsConfig,
    // Check whether message == 0 in the end
    pub ensure_no_message_left: bool,
    pub authorization_header: String,
    pub key_pair: (RsaPrivateKey, RsaPublicKey),

    has_dropped: bool,
    has_registered: bool,
    has_unregistered: bool,
}

// Utils functions implemented
impl TestUser {
    pub async fn random_readable(app: &ClientCore) -> Self {
        let name = FAKE_MANAGER.lock().generate_unique_name();
        let email = FAKE_MANAGER.lock().generate_unique_email();
        Self::new(name, email, app).await
    }

    pub async fn random_unreadable(app: &ClientCore) -> Self {
        let name = generate_random_string(25);
        let email = format!("{}@example.com", generate_random_string(25));
        Self::new(name, email, app).await
    }

    async fn new(name: impl Into<String>, email: impl Into<String>, app: &ClientCore) -> Self {
        let url = app.rpc_url.clone();
        let mut rng = rand::rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        Self {
            name: name.into(),
            password: rand::rng()
                .sample_iter(&rand::distr::Alphanumeric)
                .take(40)
                .map(char::from)
                .collect(),
            email: email.into(),
            port: app.port,
            has_dropped: false,
            clients: app.clients.clone(),
            rpc_url: url,
            // reserved
            ocid: OCID::default(),
            token: String::default(),
            oc_server: None,
            server_manage_client: None,
            id: ID::default(),
            timestamp_receive_msg: chrono::Utc::now(),
            has_unregistered: false,
            has_registered: false,
            tls: TlsConfig::default(),
            ensure_no_message_left: false,
            authorization_header: "Bearer".to_string(),
            key_pair: (private_key, public_key),
        }
    }

    pub fn public_key_bytes(&self) -> Bytes {
        self.key_pair
            .1
            .to_pkcs1_der()
            .expect("PKCS#1 serialization failed")
            .as_bytes()
            .to_vec()
            .into()
    }

    pub async fn register_internal(user: &mut TestUser) -> Result<(), ClientErr> {
        let request = RegisterRequest {
            name: user.name.clone(),
            password: user.password.clone(),
            email: user.email.clone(),
            public_key: user.public_key_bytes(),
        };
        let ret = user.clients.auth.register(request).await?.into_inner();
        user.ocid = OCID(ret.ocid);
        user.id = ID(ret.id);
        user.token = ret.token;
        let mut tls_config = None;
        if user.tls.is_tls_on()? {
            let client_cert =
                std::fs::read_to_string(user.tls.client_tls_cert_path.clone().unwrap())?;
            let client_key =
                std::fs::read_to_string(user.tls.client_key_cert_path.clone().unwrap())?;
            let client_identity = Identity::from_pem(client_cert.clone(), client_key);
            let server_ca_cert =
                std::fs::read_to_string(user.tls.ca_tls_cert_path.clone().unwrap())?;
            let server_root_ca = Certificate::from_pem(server_ca_cert);
            tls_config = Some(
                ClientTlsConfig::new()
                    .ca_certificate(server_root_ca)
                    .identity(client_identity),
            );
        }
        let channel =
            Channel::builder(Uri::from_maybe_shared(user.rpc_url.clone()).context("Uri error")?);
        let channel = if user.tls.is_tls_on()? {
            channel
                .tls_config(tls_config.unwrap())
                .context("tls config error")?
        } else {
            channel
        }
        .connect()
        .await
        .context("connect error")?;
        let token: MetadataValue<_> = format!("{} {}", user.authorization_header, user.token)
            .to_string()
            .parse()
            .context("token parse error")?;
        let token_clone = token.clone();
        user.oc_server = Some(OurChatServiceClient::with_interceptor(
            channel.clone(),
            Box::new(move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert(JWT_HEADER, token.clone());
                Ok(req)
            }),
        ));
        user.server_manage_client = Some(ServerManageServiceClient::with_interceptor(
            channel,
            Box::new(move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert(JWT_HEADER, token_clone.clone());
                Ok(req)
            }),
        ));
        user.has_registered = true;
        Ok(())
    }

    pub async fn async_drop(&mut self) {
        if !self.has_unregistered {
            if self.ensure_no_message_left {
                claims::assert_err!(self.fetch_msgs().fetch(1).await);
            }
            claims::assert_ok!(self.unregister().await);
            tracing::info!("unregister done");
        }
        self.has_dropped = true;
    }
}

// Features implemented
impl TestUser {
    pub async fn accept_join_session_invitation(
        &mut self,
        session_id: SessionID,
        accept: bool,
        inviter: ID,
    ) -> Result<(), Status> {
        let req = AcceptJoinSessionInvitationRequest {
            session_id: session_id.into(),
            accepted: accept,
            inviter_id: inviter.into(),
        };
        self.oc().accept_join_session_invitation(req).await?;
        Ok(())
    }

    pub async fn register(&mut self) -> Result<(), ClientErr> {
        Self::register_internal(self).await
    }

    pub async fn unregister(&mut self) -> tonic::Result<()> {
        let req = UnregisterRequest {};
        self.oc().unregister(req).await?;
        self.has_unregistered = true;
        Ok(())
    }

    pub fn oc(&mut self) -> &mut OCClient {
        self.oc_server.as_mut().unwrap()
    }

    pub fn server_manage(&mut self) -> &mut ServerManageClient {
        self.server_manage_client.as_mut().unwrap()
    }

    pub fn basic(&mut self) -> &mut BasicServiceClient<Channel> {
        &mut self.clients.basic
    }

    pub async fn ocid_auth(&mut self) -> Result<(), ClientErr> {
        let login_req = AuthRequest {
            account: Some(auth_request::Account::Ocid(self.ocid.0.clone())),
            password: self.password.clone(),
        };
        let ret = self.clients.auth.auth(login_req).await?.into_inner();
        self.token = ret.token.clone();
        Ok(())
    }

    pub async fn email_auth(&mut self) -> Result<(), ClientErr> {
        self.email_auth_internal(self.password.clone()).await
    }

    pub async fn email_auth_internal(
        &mut self,
        password: impl Into<String>,
    ) -> Result<(), ClientErr> {
        let login_req = AuthRequest {
            account: Some(auth_request::Account::Email(self.email.clone())),
            password: password.into(),
        };
        let ret = self.clients.auth.auth(login_req).await?.into_inner();
        assert_eq!(*self.id, ret.id);
        Ok(())
    }

    pub async fn post_file(
        &mut self,
        content: &[u8],
        session_id: Option<SessionID>,
    ) -> anyhow::Result<String> {
        self.post_file_as_iter(
            content.chunks(1024 * 1024).map(|chunk| chunk.to_vec()),
            session_id,
        )
        .await
    }

    pub async fn post_file_as_iter(
        &mut self,
        content: impl Iterator<Item = Vec<u8>> + Clone,
        session_id: Option<SessionID>,
    ) -> anyhow::Result<String> {
        use pb::service::ourchat::upload::v1::UploadRequest;
        use prost::bytes::Bytes;
        use sha3::{Digest, Sha3_256};
        let mut size = 0;
        let mut hasher = Sha3_256::new();
        for chunks in content.clone() {
            hasher.update(&chunks);
            size += chunks.len();
        }
        let hash = hasher.finalize();
        let mut files_content = vec![UploadRequest::new_header(
            size,
            #[allow(deprecated)]
            Bytes::copy_from_slice(hash.as_slice()),
            false,
            session_id.map(|x| x.0),
        )];
        for chunks in content {
            chunks.chunks(1024 * 1024).for_each(|chunk| {
                files_content.push(UploadRequest::new_content(Bytes::from(chunk.to_vec())));
            })
        }
        let ret = self.oc().upload(tokio_stream::iter(files_content)).await?;
        let ret = ret.into_inner();
        Ok(ret.key)
    }

    pub async fn download_file(
        &mut self,
        key: impl Into<String>,
    ) -> Result<Vec<u8>, tonic::Status> {
        let mut files_part = self.download_file_as_iter(key).await?;
        let mut file_download = Vec::new();
        while let Some(part) = files_part.next().await {
            let part = part?;
            file_download.extend_from_slice(&part.data);
        }
        Ok(file_download)
    }

    pub async fn download_file_as_iter(
        &mut self,
        key: impl Into<String>,
    ) -> Result<Streaming<DownloadResponse>, tonic::Status> {
        let files_part = self
            .oc()
            .download(DownloadRequest { key: key.into() })
            .await?;
        // Allow
        Ok(files_part.into_inner())
    }

    pub async fn delete_file(&mut self, key: impl Into<String>) -> Result<(), tonic::Status> {
        self.oc()
            .delete_file(DeleteFileRequest { key: key.into() })
            .await?;
        Ok(())
    }

    /// Chunked upload for gRPC-web compatibility
    pub async fn post_file_chunked(
        &mut self,
        content: &[u8],
        session_id: Option<SessionID>,
    ) -> anyhow::Result<String> {
        use sha3::{Digest, Sha3_256};
        let hash = Sha3_256::digest(content);
        let size = content.len() as u64;

        // Start upload session
        let start_response = self
            .oc()
            .start_upload(StartUploadRequest {
                hash: Bytes::from(hash.to_vec()),
                size,
                auto_clean: true,
                session_id: session_id.map(|x| x.0),
            })
            .await?
            .into_inner();

        let upload_id = start_response.upload_id;
        let chunk_size = start_response.chunk_size as usize;

        // Upload chunks sequentially
        for (chunk_id, chunk) in content.chunks(chunk_size).enumerate() {
            self.oc()
                .upload_chunk(UploadChunkRequest {
                    upload_id: upload_id.clone(),
                    chunk_data: Bytes::from(chunk.to_vec()),
                    chunk_id: chunk_id as u64,
                })
                .await?;
        }

        // Complete upload
        let complete_response = self
            .oc()
            .complete_upload(CompleteUploadRequest {
                upload_id: upload_id.clone(),
            })
            .await?
            .into_inner();

        Ok(complete_response.key)
    }

    /// Cancel an ongoing upload
    pub async fn cancel_upload(&mut self, upload_id: impl Into<String>) -> anyhow::Result<()> {
        self.oc()
            .cancel_upload(CancelUploadRequest {
                upload_id: upload_id.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn send_msg(
        &mut self,
        session_id: SessionID,
        markdown_text: impl Into<String>,
        involved_files: Vec<String>,
        is_encrypted: bool,
    ) -> Result<Response<SendMsgResponse>, ClientErr> {
        let req = SendMsgRequest {
            session_id: session_id.into(),
            is_encrypted,
            markdown_text: markdown_text.into(),
            involved_files,
        };
        Ok(self.oc().send_msg(req).await?)
    }

    /// # Warning
    /// Must request the server, shouldn't build a time from local by chrono, because some tests
    /// rely on this behavior
    pub async fn get_timestamp(&mut self) -> TimeStampUtc {
        let ret = self
            .clients
            .basic
            .timestamp(TimestampRequest {})
            .await
            .unwrap()
            .into_inner()
            .timestamp
            .unwrap();
        ret.try_into().unwrap()
    }

    pub async fn ban_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
        duration: Option<Duration>,
    ) -> Result<(), Status> {
        let req = BanUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
            duration: duration.map(|x| x.into()),
        };
        self.oc().ban_user(req).await?;
        Ok(())
    }

    pub async fn kick_user(&mut self, user_id: ID, session_id: SessionID) -> Result<(), Status> {
        let req = KickUserRequest {
            session_id: session_id.into(),
            user_id: user_id.into(),
        };
        self.oc().kick_user(req).await?;
        Ok(())
    }

    pub async fn mute_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
        duration: Option<Duration>,
    ) -> Result<(), Status> {
        let req = MuteUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
            duration: duration.map(|x| x.into()),
        };
        self.oc().mute_user(req).await?;
        Ok(())
    }

    pub async fn unban_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
    ) -> Result<(), Status> {
        let req = UnbanUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
        };
        self.oc().unban_user(req).await?;
        Ok(())
    }

    pub async fn unmute_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
    ) -> Result<(), Status> {
        let req = UnmuteUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
        };
        self.oc().unmute_user(req).await?;
        Ok(())
    }

    pub fn fetch_msgs(&mut self) -> FetchMsgBuilder<'_> {
        let tmp = self.timestamp_receive_msg;
        FetchMsgBuilder {
            user: self,
            timestamp: tmp,
            timeout_limit: DEFAULT_FETCH_TIMEOUT_LIMIT,
        }
    }

    pub async fn get_self_info(
        &mut self,
        queried_values: Vec<get_account_info::v1::QueryValues>,
    ) -> anyhow::Result<GetAccountInfoResponse> {
        let id = self.id;
        self.get_account_info(id, queried_values).await
    }

    pub async fn get_account_info(
        &mut self,
        id: ID,
        queried_values: Vec<get_account_info::v1::QueryValues>,
    ) -> anyhow::Result<GetAccountInfoResponse> {
        Ok(self
            .oc()
            .get_account_info(GetAccountInfoRequest {
                id: Some(id.into()),
                request_values: queried_values.into_iter().map(|x| x.into()).collect(),
            })
            .await?
            .into_inner())
    }

    pub async fn get_update_timestamp(&mut self) -> anyhow::Result<TimeStampUtc> {
        self.get_self_info(vec![get_account_info::v1::QueryValues::UpdatedTime])
            .await?
            .updated_time
            .unwrap()
            .try_into()
    }

    /// Promote this user to an admin (assigns Admin role) by directly modifying the database.
    /// This bypasses the permission check, solving the chicken-and-egg problem where
    /// the first admin needs AssignRole permission to assign themselves as admin.
    ///
    /// # Arguments
    /// * `db` - The database connection to use for the role assignment
    pub async fn promote_to_admin(
        &mut self,
        db: &sea_orm::DatabaseConnection,
    ) -> Result<(), Status> {
        server::db::manager::set_role(self.id, PredefinedServerManagementRole::Admin as i64, db)
            .await
            .map_err(|e| Status::internal(format!("failed to set role: {e:?}")))
    }

    /// Remove admin role from this user
    pub async fn remove_admin_role(&mut self) -> Result<(), Status> {
        let request = RemoveServerRoleRequest {
            user_id: self.id.into(),
            role_id: PredefinedServerManagementRole::Admin as u64,
        };
        self.server_manage_client
            .as_mut()
            .unwrap()
            .remove_server_role(request)
            .await?;
        Ok(())
    }
}

impl Drop for TestUser {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() && self.has_registered {
            panic!("async_drop is not called to drop this user");
        }
    }
}

pub type TestUserShared = Arc<tokio::sync::Mutex<TestUser>>;
const DEFAULT_FETCH_TIMEOUT_LIMIT: Duration = Duration::from_secs(20);

pub struct FetchMsgBuilder<'a> {
    pub timestamp: TimeStampUtc,
    user: &'a mut TestUser,
    timeout_limit: Duration,
}

#[derive(thiserror::Error, Debug)]
pub enum FetchMsgErr {
    #[error("time limit exceeded")]
    Timeout(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<'a> FetchMsgBuilder<'a> {
    pub async fn fetch(
        &mut self,
        nums_limit: usize,
    ) -> Result<Vec<FetchMsgsResponse>, FetchMsgErr> {
        let msg_get = FetchMsgsRequest {
            time: Some(self.timestamp.into()),
            announcement_only: false,
        };
        tracing::info!("timestamp_receive_msg: {}", self.timestamp);
        let ret = self
            .user
            .oc()
            .fetch_msgs(msg_get)
            .await
            .context("error from server side")?;
        let mut ret_stream = ret.into_inner();
        let msgs = Arc::new(Mutex::new(vec![]));
        let msgs_clone = msgs.clone();
        let logic = async {
            while let Some(i) = ret_stream.next().await {
                let i = i?;
                self.user.timestamp_receive_msg = i.time.unwrap().try_into().unwrap();
                let mut msgs = msgs_clone.lock().await;
                msgs.push(i);
                if msgs.len() == nums_limit {
                    break;
                }
            }
            anyhow::Ok(())
        };
        select! {
            err = logic => {
                err?;
                Ok(msgs.lock().await.to_vec())
            }
            _ = tokio::time::sleep(self.timeout_limit) => {
                let lock = msgs.lock().await;
                Err(FetchMsgErr::Timeout(format!("Received {} messages: {:?}", lock.len(), lock)))
            }
        }
    }

    pub async fn fetch_with_notify(
        &mut self,
        notify: Arc<Notify>,
    ) -> Result<Vec<FetchMsgsResponse>, Status> {
        let msg_get = FetchMsgsRequest {
            time: Some(self.timestamp.into()),
            announcement_only: false,
        };
        let ret = self.user.oc().fetch_msgs(msg_get).await?;
        let mut ret_stream = ret.into_inner();
        let mut msgs = vec![];
        let logic = async {
            while let Some(i) = ret_stream.next().await {
                let i = i?;
                self.user.timestamp_receive_msg = i.time.unwrap().try_into().unwrap();
                msgs.push(i);
            }
            Result::<_, Status>::Ok(())
        };
        select! {
            _ = logic => {},
            _ = notify.notified() => {}
        }
        Ok(msgs)
    }

    pub fn set_timestamp(mut self, timestamp: TimeStampUtc) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn set_timeout(mut self, timeout_limit: Duration) -> Self {
        self.timeout_limit = timeout_limit;
        self
    }
}
