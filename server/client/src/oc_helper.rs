use base::consts::SessionID;
use bytes::Bytes;
use fake::Fake;
use fake::faker::internet::raw::FreeEmail;
use fake::faker::name::en;
use fake::faker::name::raw::Name;
use fake::locales::EN;
use parking_lot::Mutex;
use pb::service::auth::v1::auth_service_client::AuthServiceClient;
use pb::service::basic::v1::basic_service_client::BasicServiceClient;
use pb::service::server_manage::v1::server_manage_service_client::ServerManageServiceClient;
use ring::rand::{SecureRandom as _, SystemRandom};
use std::collections::HashSet;
use std::sync::LazyLock;
use tonic::transport::Channel;

pub mod client;
pub mod user;

#[derive(Debug, thiserror::Error)]
pub enum ClientErr {
    #[error("rpc status:{0}")]
    RpcStatus(#[from] tonic::Status),
    #[error("io error:{0}")]
    IoError(#[from] std::io::Error),
    #[error("unknown error:{0}")]
    Unknown(#[from] anyhow::Error),
}

impl ClientErr {
    pub fn unwrap_rpc_status(self) -> tonic::Status {
        match self {
            Self::RpcStatus(status) => status,
            Self::IoError(e) => panic!("expect rpc status, but got io error: {e}"),
            Self::Unknown(e) => panic!("expect rpc status, but got unknown error: {e}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clients {
    pub auth: AuthServiceClient<Channel>,
    pub basic: BasicServiceClient<Channel>,
    pub server_manage: ServerManageServiceClient<Channel>,
}

struct FakeManager {
    dup_name: HashSet<String>,
    dup_email: HashSet<String>,
    name_faker: Name<EN>,
    email_faker: FreeEmail<EN>,
}

impl FakeManager {
    fn new() -> Self {
        Self {
            dup_name: HashSet::new(),
            name_faker: en::Name(),
            dup_email: HashSet::new(),
            email_faker: fake::faker::internet::en::FreeEmail(),
        }
    }

    fn generate_unique_name(&mut self) -> String {
        loop {
            let name: String = self.name_faker.fake();
            if !self.dup_name.contains(&name) {
                self.dup_name.insert(name.clone());
                return name;
            }
        }
    }

    fn generate_unique_email(&mut self) -> String {
        loop {
            let email: String = self.email_faker.fake();
            if !self.dup_email.contains(&email) {
                self.dup_email.insert(email.clone());
                return email;
            }
        }
    }
}

static FAKE_MANAGER: LazyLock<Mutex<FakeManager>> =
    LazyLock::new(|| Mutex::new(FakeManager::new()));

pub struct TestSession {
    pub session_id: SessionID,
    pub room_key: Option<String>,
}

impl TestSession {
    pub fn new(session_id: SessionID) -> Self {
        Self {
            session_id,
            room_key: None,
        }
    }

    pub fn generate_room_key() -> Bytes {
        let mut key = vec![0u8; 32];
        SystemRandom::new().fill(key.as_mut_slice()).unwrap();
        key.into()
    }
}
