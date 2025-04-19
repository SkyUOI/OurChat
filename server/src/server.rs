//! OurChat Server

mod ourchat_service;

use crate::db::user::get_account_info_db;
use crate::process::basic::get_preset_user_status::get_preset_user_status;
use crate::process::basic::support::support;
use crate::process::db::get_id;
use crate::process::error_msg::{self, ACCOUNT_DELETED, SERVER_ERROR};
use crate::process::{self, ErrAuth};
use crate::{SERVER_INFO, SharedData, ShutdownRev};
use base::consts::{ID, OCID, VERSION_SPLIT};
use base::database::DbPool;
use base::time::to_google_timestamp;
use http_body_util::BodyExt;
use http_body_util::Full;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;
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
    GetIdRequest, GetIdResponse, GetServerInfoRequest, TimestampRequest, TimestampResponse,
};
use pb::service::ourchat::v1::our_chat_service_server::OurChatServiceServer;
use pb::service::server_manage::delete_account::v1::{DeleteAccountRequest, DeleteAccountResponse};
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
use tonic::service::Routes;
use tonic::{Request, Response, Status};
use tower::Service;
use tower::ServiceExt;
use tracing::info;

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
}

/// Check if the request is a gRPC request by examining the content-type header
fn is_grpc_request(req: &hyper::Request<impl hyper::body::Body>) -> bool {
    req.headers()
        .get("content-type")
        .map(|v| v.as_bytes().starts_with(b"application/grpc"))
        .unwrap_or(false)
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
        };

        // Clone shared data for service interceptors
        let shared_data = self.shared_data.clone();
        let shared_data1 = self.shared_data.clone();
        let shared_data2 = self.shared_data.clone();
        let shared_data3 = self.shared_data.clone();

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
        let test = routes.prepare();
        let grpc_server = tower::ServiceBuilder::new().service(test);

        // Main server loop
        let server = async move {
            let listener = TcpListener::bind(addr).await?;
            loop {
                // Accept incoming connections
                let (socket, _) = listener.accept().await?;
                let io = TokioIo::new(socket);
                let svc = grpc_server.clone();

                // Spawn a new task for each connection
                tokio::spawn(async move {
                    let service = hyper::service::service_fn(
                        move |req: hyper::Request<hyper::body::Incoming>| {
                            let svc = svc.clone();
                            async move {
                                // Convert incoming request to appropriate format
                                let (parts, body) = req.into_parts();
                                let body = UnsyncBoxBody::<Bytes, Status>::new(
                                    body.map_err(|_| Status::internal("Body error")),
                                );
                                let converted_req =
                                    hyper::Request::from_parts(parts, tonic::body::Body::new(body));

                                // Handle gRPC and non-gRPC requests differently
                                if is_grpc_request(&converted_req) {
                                    // Process gRPC request
                                    let mut svc = tower::util::MapRequest::new(
                                        svc.clone(),
                                        |req: hyper::Request<tonic::body::Body>| req,
                                    );
                                    match svc.ready().await {
                                        Ok(service) => {
                                            // Handle successful service call
                                            let resp = service.call(converted_req).await;
                                            match resp {
                                                Ok(res) => Ok(res),
                                                Err(e) => {
                                                    let body = Full::new(Bytes::from(format!(
                                                        "Error: {}",
                                                        e
                                                    )))
                                                    .map_err(|_| Status::internal("Body error"))
                                                    .boxed_unsync();
                                                    Ok::<_, Status>(
                                                        hyper::Response::builder()
                                                            .status(500)
                                                            .header("content-type", "text/plain")
                                                            .body(tonic::body::Body::new(body))
                                                            .unwrap(),
                                                    )
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            // Handle service unavailable
                                            let body =
                                                Full::new(Bytes::from("Service unavailable"))
                                                    .map_err(|_| Status::internal("Body error"))
                                                    .boxed_unsync();
                                            Ok::<_, Status>(
                                                hyper::Response::builder()
                                                    .status(503)
                                                    .body(tonic::body::Body::new(body))
                                                    .unwrap(),
                                            )
                                        }
                                    }
                                } else {
                                    // Handle non-gRPC request with 404
                                    let body = Full::new(Bytes::from("Not implemented"))
                                        .map_err(|_| Status::internal("Body error"))
                                        .boxed_unsync();
                                    Ok::<_, Status>(
                                        hyper::Response::builder()
                                            .status(404)
                                            .body(tonic::body::Body::new(body))
                                            .unwrap(),
                                    )
                                }
                            }
                        },
                    );

                    // Serve HTTP/2 connection
                    if let Err(err) = http2::Builder::new(hyper_util::rt::TokioExecutor::default())
                        .serve_connection(io, service)
                        .await
                    {
                        tracing::error!("Connection error: {:?}", err);
                    }
                });
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
        match req.metadata().get("token") {
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
            timestamp: Some(to_google_timestamp(time)),
        };
        Ok(Response::new(res))
    }

    /// Get server information including a version, status and configuration
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
