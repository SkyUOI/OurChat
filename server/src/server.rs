//! OurChat Server

mod ourchat_service;

use crate::db::user::get_account_info_db;
use crate::process::basic::get_preset_user_status::get_preset_user_status;
use crate::process::basic::support::support;
use crate::process::db::get_id;
use crate::process::error_msg::{self, ACCOUNT_DELETED, SERVER_ERROR};
use crate::process::{self, ErrAuth};
use crate::{SERVER_INFO, SharedData, ShutdownRev};
use base::consts::{ID, JWT_HEADER, OCID, VERSION_SPLIT};
use base::database::DbPool;
use futures_util::future::BoxFuture;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::rt::{Read, Write};
use hyper::server::conn::http2;
use hyper_util::rt::TokioIo;
use migration::m20250301_005919_add_soft_delete_columns::AccountStatus;
use pb::service::auth::authorize::v1::{AuthRequest, AuthResponse};
use pb::service::auth::email_verify::v1::{VerifyRequest, VerifyResponse};
use pb::service::auth::register::v1::{RegisterRequest, RegisterResponse};
use pb::service::auth::v1::auth_service_server::{self, AuthServiceServer};
use pb::service::basic::preset_user_status::v1::{
    GetPresetUserStatusRequest, GetPresetUserStatusResponse,
};
use pb::service::basic::server::v1::RunningStatus;
use pb::service::basic::support::v1::{SupportRequest, SupportResponse};
use pb::service::basic::v1::basic_service_server::{BasicService, BasicServiceServer};
use pb::service::basic::v1::{
    GetIdRequest, GetIdResponse, GetServerInfoRequest, PingRequest, PingResponse, TimestampRequest,
    TimestampResponse,
};
use pb::service::ourchat::v1::our_chat_service_server::OurChatServiceServer;
use pb::service::server_manage::delete_account::v1::{DeleteAccountRequest, DeleteAccountResponse};
use pb::service::server_manage::publish_announcement::v1::{
    PublishAnnouncementRequest, PublishAnnouncementResponse,
};
use pb::service::server_manage::set_server_status::v1::{
    SetServerStatusRequest, SetServerStatusResponse,
};
use pb::service::server_manage::v1::server_manage_service_server::{
    ServerManageService, ServerManageServiceServer,
};
use process::error_msg::not_found;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use tokio::net::TcpListener;
use tokio::select;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::pki_types::PrivateKeyDer;
use tokio_rustls::rustls::pki_types::{CertificateDer, pem::PemObject as _};
use tokio_rustls::rustls::server::WebPkiClientVerifier;
use tokio_rustls::rustls::{RootCertStore, ServerConfig};
use tonic::service::Routes;
use tonic::{Request, Response, Status};
use tower::Service;
use tracing::{debug, info, warn};

pub use ourchat_service::*;

/// RPC Server implementation for OurChat
/// Handles all service requests and manages connections
#[derive(Debug)]
pub struct RpcServer {
    pub db: DbPool,
    pub shared_data: Arc<SharedData>,
    pub addr: SocketAddr,
    pub rabbitmq: deadpool_lapin::Pool,
}

/// Server management service provider
pub struct ServerManageServiceProvider {
    pub shared_data: Arc<SharedData>,
    pub db: DbPool,
    pub rabbitmq: deadpool_lapin::Pool,
}

/// Check if the request is a gRPC request by examining the content-type header
fn is_grpc_request(req: &hyper::Request<impl hyper::body::Body>) -> bool {
    req.headers()
        .get("content-type")
        .map(|v| v.as_bytes().starts_with(b"application/grpc"))
        .unwrap_or(false)
}

fn launch_connection<B>(svc: B, io: impl Read + Write + Unpin + Send + 'static)
where
    B: Service<
            hyper::Request<hyper::body::Incoming>,
            Response = hyper::Response<tonic::body::Body>,
            Error = std::convert::Infallible,
        > + Clone
        + Send
        + 'static,
    B::Future: Send,
{
    tokio::spawn(async move {
        if let Err(err) = http2::Builder::new(hyper_util::rt::TokioExecutor::default())
            .serve_connection(
                io,
                hyper::service::service_fn(
                    move |req| -> BoxFuture<
                        'static,
                        Result<hyper::Response<tonic::body::Body>, std::convert::Infallible>,
                    > {
                        let mut inner_svc = svc.clone();
                        Box::pin(async move {
                            if is_grpc_request(&req) {
                                inner_svc.call(req).await
                            } else {
                                let body = Full::new(Bytes::from("Not implemented"))
                                    .map_err(|_| Status::internal("Body error"))
                                    .boxed_unsync();
                                Ok(hyper::Response::builder()
                                    .status(404)
                                    .body(tonic::body::Body::new(body))
                                    .unwrap())
                            }
                        })
                    },
                ),
            )
            .await
        {
            tracing::error!("Connection error: {:?}", err);
        }
    });
}

impl RpcServer {
    /// Create a new RPC server instance
    ///
    /// # Arguments
    /// * `ip` - Server address to bind to
    /// * `db` - Database connection pool
    /// * `shared_data` - Shared server data
    /// * `rabbitmq` - RabbitMQ connection pool
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

    /// Start the RPC server and listen for connections
    ///
    /// # Arguments
    /// * `shutdown_rev` - Shutdown receiver to gracefully stop the server
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn run(self, mut shutdown_rev: ShutdownRev) -> anyhow::Result<()> {
        // Log server startup
        info!("starting rpc server, connecting to {}", self.addr);
        let addr = self.addr;

        // Initialize service providers with shared resources
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
            shared_data: self.shared_data.clone(),
            db: self.db.clone(),
            rabbitmq: self.rabbitmq.clone(),
        };

        // Clone shared data for service interceptors
        let shared_data = self.shared_data.clone();
        let shared_data1 = self.shared_data.clone();
        let shared_data2 = self.shared_data.clone();
        let shared_data3 = self.shared_data.clone();
        let shared_data_for_tls = self.shared_data.clone();

        // Create service instances with interceptors for authentication and maintenance checks
        let main_svc = OurChatServiceServer::with_interceptor(self, move |mut req| {
            // Check if server is in maintenance mode
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
            ServerManageServiceServer::with_interceptor(server_manage_service, move |mut req| {
                shared_data3.convert_maintaining_into_grpc_status()?;
                Self::check_auth(&mut req)?;
                Ok(req)
            });

        // Build the gRPC router with all services
        let mut builder = Routes::builder();
        builder.add_service(main_svc);
        builder.add_service(basic_svc);
        builder.add_service(auth_svc);
        builder.add_service(server_manage_svc);
        let routes = builder.routes();
        let svc = routes.prepare();

        let cert_path = shared_data_for_tls
            .cfg
            .main_cfg
            .tls
            .server_tls_cert_path
            .clone();
        let key_path = shared_data_for_tls
            .cfg
            .main_cfg
            .tls
            .server_key_cert_path
            .clone();
        let client_ca_cert_path = shared_data_for_tls
            .cfg
            .main_cfg
            .tls
            .client_ca_tls_cert_path
            .clone();
        let is_tls_on = shared_data_for_tls.cfg.main_cfg.tls.is_tls_on()?;
        let client_certificate_required = shared_data_for_tls
            .cfg
            .main_cfg
            .tls
            .client_certificate_required;

        let mut tls_acceptor = None;
        if is_tls_on {
            let cert_path = cert_path.unwrap();
            let key_path = key_path.unwrap();
            let client_ca_cert_path = client_ca_cert_path.unwrap();
            info!(
                "TLS on: cert_path = {}, key_path = {}",
                cert_path.display(),
                key_path.display(),
            );
            let certs = {
                let fd = std::fs::File::open(cert_path)?;
                let mut buf = std::io::BufReader::new(&fd);
                CertificateDer::pem_reader_iter(&mut buf).collect::<Result<Vec<_>, _>>()?
            };
            let key = {
                let fd = std::fs::File::open(key_path)?;
                let mut buf = std::io::BufReader::new(&fd);
                PrivateKeyDer::from_pem_reader(&mut buf)?
            };
            let client_ca_cert = {
                let fd = std::fs::File::open(client_ca_cert_path)?;
                let mut buf = std::io::BufReader::new(&fd);
                CertificateDer::pem_reader_iter(&mut buf).collect::<Result<Vec<_>, _>>()?
            };

            let mut client_root_store = RootCertStore::empty();
            for cert in &client_ca_cert {
                client_root_store.add(cert.clone()).map_err(|e| {
                    anyhow::anyhow!("Failed to add certificate to RootCertStore: {:?}", e)
                })?;
            }

            let client_cert = WebPkiClientVerifier::builder(Arc::new(client_root_store));

            let mut tls = if client_certificate_required {
                ServerConfig::builder().with_client_cert_verifier(client_cert.build()?)
            } else {
                ServerConfig::builder().with_no_client_auth()
            }
            .with_single_cert(certs, key)?;

            tls.alpn_protocols = vec![b"h2".to_vec()];
            tls_acceptor = Some(TlsAcceptor::from(Arc::new(tls)));
            debug!("TLS enabled: {}", tls_acceptor.is_some());
        } else {
            warn!("TLS disabled, this is insecure.");
        }

        // Main server loop
        let server = async move {
            let listener = TcpListener::bind(addr).await?;
            loop {
                // Accept incoming connections
                let (socket, _) = listener.accept().await?;

                let tls_io;
                let io;
                let tls_acceptor = tls_acceptor.clone();
                let svc_routes = svc.clone();

                if is_tls_on {
                    debug!("tls_io getting");
                    tls_io = TokioIo::new(tls_acceptor.unwrap().accept(socket).await?);
                    let svc = tower::ServiceBuilder::new().service(svc_routes.clone());
                    launch_connection(svc, tls_io);
                } else {
                    io = TokioIo::new(socket);
                    let svc = tower::ServiceBuilder::new().service(svc_routes);
                    launch_connection(svc, io);
                }
                debug!("execute after accepting");
            }
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        };

        // Handle shutdown signal or server error
        select! {
            _ = shutdown_rev.wait_shutting_down() => {}
            err = server => {
                tracing::error!("Server main loop error: {:?}", err);
                err?
            }
        }
        Ok(())
    }

    /// Verify authentication token from request metadata and extract user ID
    ///
    /// # Arguments
    /// * `req` - The request to check authentication for
    ///
    /// # Returns
    /// * `Ok(ID)` - The authenticated user's ID
    /// * `Err(Status)` - Authentication error status
    #[allow(clippy::result_large_err)]
    fn check_auth(req: &mut Request<()>) -> Result<ID, Status> {
        // Check if token exists in metadata
        match req.metadata().get(JWT_HEADER) {
            Some(token) => {
                match process::check_token(token.to_str().unwrap()) {
                    Ok(jwt) => {
                        let ret = jwt.id;
                        // Store user ID in request metadata for later use
                        req.metadata_mut()
                            .insert("id", jwt.id.to_string().parse().unwrap());
                        Ok(ret)
                    }
                    Err(e) => match e {
                        ErrAuth::JWT(_) => Err(Status::unauthenticated(error_msg::token::INVALID)),
                        ErrAuth::Expire => Err(Status::unauthenticated(error_msg::token::EXPIRED)),
                        ErrAuth::UnsupportedAuthorizationHeader => Err(Status::unauthenticated(
                            error_msg::token::UNSUPPORTED_AUTHORIZATION_HEADER,
                        )),
                        ErrAuth::IncorrectFormat => {
                            Err(Status::unauthenticated(error_msg::token::INCORRECT_FORMAT))
                        }
                    },
                }
            }
            None => Err(Status::unauthenticated(error_msg::token::MISSING)),
        }
    }

    /// Check if the user account exists and is not deleted
    ///
    /// # Arguments
    /// * `id` - User ID to check
    ///
    /// # Returns
    /// * `Ok(())` - Account exists and is active
    /// * `Err(Status)` - Account not found or deleted
    async fn check_account_status(&self, id: ID) -> Result<(), Status> {
        let account = match get_account_info_db(id, &self.db.db_pool)
            .await
            .map_err(|_| Status::internal(SERVER_ERROR))?
        {
            Some(account) => account,
            None => return Err(Status::unauthenticated(not_found::USER)),
        };

        // Return error if the account has been deleted
        if account.account_status == AccountStatus::Deleted as i32 {
            return Err(Status::unauthenticated(ACCOUNT_DELETED));
        }

        Ok(())
    }
}

/// Authentication service provider
#[derive(Debug)]
pub struct AuthServiceProvider {
    pub shared_data: Arc<SharedData>,
    pub db: DbPool,
    pub rabbitmq: deadpool_lapin::Pool,
}

/// Stream type for verification responses
pub type VerifyStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<VerifyResponse, Status>> + Send>>;

/// Implementation of Authentication service methods
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

/// Basic service implementation providing server information and utilities
#[derive(Debug)]
pub struct BasicServiceProvider {
    pub shared_data: Arc<SharedData>,
    pub db: DbPool,
}

/// Implementation of Basic service methods
#[tonic::async_trait]
impl BasicService for BasicServiceProvider {
    /// Get current server timestamp in UTC
    #[tracing::instrument(skip(self))]
    async fn timestamp(
        &self,
        _request: Request<TimestampRequest>,
    ) -> Result<Response<TimestampResponse>, Status> {
        // Return current UTC timestamp
        let time = chrono::Utc::now();
        let res = TimestampResponse {
            timestamp: Some(time.into()),
        };
        Ok(Response::new(res))
    }

    /// Get server information including the version, status and configuration
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

    /// Convert OCID to internal user ID
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

    /// Handle support requests
    async fn support(
        &self,
        request: Request<SupportRequest>,
    ) -> Result<Response<SupportResponse>, Status> {
        support(self, request).await
    }

    async fn get_preset_user_status(
        &self,
        request: Request<GetPresetUserStatusRequest>,
    ) -> Result<Response<GetPresetUserStatusResponse>, Status> {
        get_preset_user_status(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn ping(&self, _request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse {}))
    }
}

// Static server information initialized at startup
// Contains version, name, and other immutable server properties
static SERVER_INFO_RPC: LazyLock<pb::service::basic::server::v1::GetServerInfoResponse> =
    LazyLock::new(|| pb::service::basic::server::v1::GetServerInfoResponse {
        server_version: Some(*VERSION_SPLIT),
        http_port: 0, // Port number set dynamically at runtime
        status: RunningStatus::Normal as i32,
        unique_identifier: SERVER_INFO.unique_id.to_string(),
        server_name: SERVER_INFO.server_name.to_string(),
    });

/// Server management service implementation
/// Provides administrative functions like account deletion
#[tonic::async_trait]
impl ServerManageService for ServerManageServiceProvider {
    /// Permanently delete a user account
    #[tracing::instrument(skip(self))]
    async fn delete_account(
        &self,
        request: Request<DeleteAccountRequest>,
    ) -> Result<Response<DeleteAccountResponse>, Status> {
        process::delete_account(self, request).await
    }

    #[tracing::instrument(skip(self))]
    async fn set_server_status(
        &self,
        request: Request<SetServerStatusRequest>,
    ) -> Result<Response<SetServerStatusResponse>, Status> {
        process::set_server_status(self, request).await
    }
    async fn publish_announcement(
        &self,
        request: Request<PublishAnnouncementRequest>,
    ) -> Result<Response<PublishAnnouncementResponse>, Status> {
        process::publish_announcement(self, request).await
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
