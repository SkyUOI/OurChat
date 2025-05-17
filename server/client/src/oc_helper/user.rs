use crate::oc_helper::FAKE_MANAGER;
use crate::oc_helper::client::{OCClient, TestApp};
use crate::oc_helper::{ClientErr, Clients};
use anyhow::Context;
use base::consts::{ID, OCID, SessionID};
use pb::service::auth::authorize::v1::{AuthRequest, auth_request};
use pb::service::auth::register::v1::RegisterRequest;
use pb::service::basic::v1::TimestampRequest;
use pb::service::ourchat::download::v1::{DownloadRequest, DownloadResponse};
use pb::service::ourchat::msg_delivery::v1::{
    BundleMsgs, FetchMsgsRequest, FetchMsgsResponse, SendMsgRequest, SendMsgResponse,
};
use pb::service::ourchat::session::accept_session::v1::AcceptSessionRequest;
use pb::service::ourchat::session::ban::v1::{BanUserRequest, UnbanUserRequest};
use pb::service::ourchat::session::mute::v1::{MuteUserRequest, UnmuteUserRequest};
use pb::service::ourchat::unregister::v1::UnregisterRequest;
use pb::service::ourchat::v1::our_chat_service_client::OurChatServiceClient;
use pb::time::{
    TimeStampUtc, from_google_timestamp, std_duration_to_prost_duration, to_google_timestamp,
};
use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::select;
use tokio::sync::Notify;
use tokio_stream::StreamExt;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Uri};
use tonic::{Response, Streaming};

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
    pub timestamp_receive_msg: TimeStampUtc,

    has_dropped: bool,
    has_registered: bool,
    has_unregistered: bool,
}

// Utils functions implemented
impl TestUser {
    pub async fn random(app: &TestApp) -> Self {
        let name = FAKE_MANAGER.lock().generate_unique_name();
        let email = FAKE_MANAGER.lock().generate_unique_email();
        let url = app.rpc_url.clone();
        Self {
            name,
            password: rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(40)
                .map(char::from)
                .collect(),
            email,
            port: app.port,
            has_dropped: false,
            clients: app.clients.clone(),
            rpc_url: url,
            // reserved
            ocid: OCID::default(),
            token: String::default(),
            oc_server: None,
            id: ID::default(),
            timestamp_receive_msg: chrono::Utc::now(),
            has_unregistered: false,
            has_registered: false,
        }
    }

    pub async fn register_internal(user: &mut TestUser) -> Result<(), ClientErr> {
        let request = RegisterRequest {
            name: user.name.clone(),
            password: user.password.clone(),
            email: user.email.clone(),
        };
        let ret = user.clients.auth.register(request).await?.into_inner();
        user.ocid = OCID(ret.ocid);
        user.id = ID(ret.id);
        user.token = ret.token;
        let channel =
            Channel::builder(Uri::from_maybe_shared(user.rpc_url.clone()).context("Uri error")?)
                .connect()
                .await
                .context("connect error")?;
        let token: MetadataValue<_> = user
            .token
            .to_string()
            .parse()
            .context("token parse error")?;
        user.oc_server = Some(OurChatServiceClient::with_interceptor(
            channel,
            Box::new(move |mut req: tonic::Request<()>| {
                req.metadata_mut().insert("token", token.clone());
                Ok(req)
            }),
        ));
        user.has_registered = true;
        Ok(())
    }

    pub(crate) async fn async_drop(&mut self) {
        if !self.has_unregistered {
            claims::assert_ok!(self.unregister().await);
            tracing::info!("unregister done");
        }
        self.has_dropped = true;
    }
}

// Features implemented
impl TestUser {
    pub async fn accept_session(
        &mut self,
        session_id: SessionID,
        accept: bool,
    ) -> anyhow::Result<()> {
        let req = AcceptSessionRequest {
            session_id: session_id.into(),
            accepted: accept,
        };
        self.oc().accept_session(req).await?;
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

    pub async fn post_file(&mut self, content: String) -> anyhow::Result<String> {
        self.post_file_as_iter(
            content
                .as_bytes()
                .chunks(1024 * 1024)
                .map(|chunk| chunk.to_vec()),
        )
        .await
    }

    pub async fn post_file_as_iter(
        &mut self,
        content: impl Iterator<Item = Vec<u8>> + Clone,
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
        let hash = format!("{hash:x}");
        let mut files_content = vec![UploadRequest::new_header(size, hash, false)];
        for chunks in content {
            chunks.chunks(1024 * 1024).for_each(|chunk| {
                files_content.push(UploadRequest::new_content(Bytes::from(chunk.to_vec())));
            })
        }
        let ret = self.oc().upload(tokio_stream::iter(files_content)).await?;
        let ret = ret.into_inner();
        Ok(ret.key)
    }

    pub async fn download_file(&mut self, key: impl Into<String>) -> anyhow::Result<Vec<u8>> {
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
    ) -> anyhow::Result<Streaming<DownloadResponse>> {
        let files_part = self
            .oc()
            .download(DownloadRequest { key: key.into() })
            .await?;
        // Allow
        Ok(files_part.into_inner())
    }

    pub async fn send_msg(
        &mut self,
        session_id: ID,
        msg: BundleMsgs,
    ) -> Result<Response<SendMsgResponse>, ClientErr> {
        let req = SendMsgRequest {
            session_id: session_id.into(),
            is_encrypted: false,
            bundle_msgs: msg,
            time: Some(to_google_timestamp(self.get_timestamp().await)),
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
        from_google_timestamp(&ret).unwrap()
    }

    pub async fn ban_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
        duration: Option<Duration>,
    ) -> Result<(), tonic::Status> {
        let req = BanUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
            duration: duration.map(std_duration_to_prost_duration),
        };
        self.oc().ban_user(req).await?;
        Ok(())
    }

    pub async fn mute_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
        duration: Option<Duration>,
    ) -> Result<(), tonic::Status> {
        let req = MuteUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
            duration: duration.map(std_duration_to_prost_duration),
        };
        self.oc().mute_user(req).await?;
        Ok(())
    }

    pub async fn unban_user(
        &mut self,
        user_ids: Vec<ID>,
        session_id: SessionID,
    ) -> Result<(), tonic::Status> {
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
    ) -> Result<(), tonic::Status> {
        let req = UnmuteUserRequest {
            user_ids: user_ids.into_iter().map(|x| x.into()).collect(),
            session_id: session_id.into(),
        };
        self.oc().unmute_user(req).await?;
        Ok(())
    }

    pub async fn fetch_msgs(
        &mut self,
        duration: Duration,
    ) -> Result<Vec<FetchMsgsResponse>, tonic::Status> {
        let msg_get = FetchMsgsRequest {
            time: Some(to_google_timestamp(self.timestamp_receive_msg)),
        };
        tracing::info!("timestamp_receive_msg: {}", self.timestamp_receive_msg);
        let ret = self.oc().fetch_msgs(msg_get).await?;
        let mut ret_stream = ret.into_inner();
        let mut msgs = vec![];
        while let Some(i) = select! {
            i = ret_stream.next() => i,
            _ = tokio::time::sleep(duration) => None
        } {
            let i = i?;
            self.timestamp_receive_msg = from_google_timestamp(&i.time.unwrap()).unwrap();
            msgs.push(i);
        }
        Ok(msgs)
    }
    pub async fn fetch_msgs_notify(
        &mut self,
        notify: Arc<Notify>,
    ) -> Result<Vec<FetchMsgsResponse>, tonic::Status> {
        let msg_get = FetchMsgsRequest {
            time: Some(to_google_timestamp(self.timestamp_receive_msg)),
        };
        let ret = self.oc().fetch_msgs(msg_get).await?;
        let mut ret_stream = ret.into_inner();
        let mut msgs = vec![];
        let logic = async {
            while let Some(i) = ret_stream.next().await {
                let i = i?;
                self.timestamp_receive_msg = from_google_timestamp(&i.time.unwrap()).unwrap();
                msgs.push(i);
            }
            Result::<_, tonic::Status>::Ok(())
        };
        select! {
            _ = logic => {},
            _ = notify.notified() => {}
        }
        Ok(msgs)
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
