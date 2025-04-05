use crate::TestApp;
use anyhow::Context;
use base::consts::ID;
use base::database::DbPool;
use pb::service::server_manage::v1::server_manage_service_client::ServerManageServiceClient;
use server::db::manager;
use tonic::Request;
use tonic::codegen::InterceptedService;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Uri};

type ServerManagerClient = ServerManageServiceClient<
    InterceptedService<
        Channel,
        Box<dyn FnMut(Request<()>) -> Result<Request<()>, tonic::Status> + Send + Sync>,
    >,
>;

pub struct TestServerManager {
    pub client: ServerManagerClient,
    pub token: String,
    pub user_id: ID,
    db_conn: DbPool,
}

impl TestServerManager {
    pub async fn new(app: &TestApp, user_id: ID, token: String) -> anyhow::Result<Self> {
        let channel =
            Channel::builder(Uri::from_maybe_shared(app.rpc_url.clone()).context("Uri Error")?)
                .connect()
                .await
                .context("Connect Error")?;
        let token_clone: MetadataValue<_> = token.parse().context("token parse error")?;
        let client: ServerManagerClient = ServerManageServiceClient::with_interceptor(
            channel,
            Box::new(move |mut req: Request<()>| {
                req.metadata_mut().insert("token", token_clone.clone());
                Ok(req)
            }),
        );
        Ok(Self {
            client,
            token,
            user_id,
            db_conn: app.db_pool.clone().unwrap(),
        })
    }

    pub async fn assign_role(&self, role_id: i64) -> anyhow::Result<()> {
        manager::set_role(self.user_id, role_id, &self.db_conn.db_pool).await?;
        Ok(())
    }
}
