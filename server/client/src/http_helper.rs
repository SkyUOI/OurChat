use crate::helper::rabbitmq::{create_random_vhost, delete_vhost};
use base::{email_client::EmailSender, shutdown::ShutdownSdr};
use http_server::{Cfg, Launcher};
use sqlx::migrate::MigrateDatabase;
use std::{sync::Arc, thread, time::Duration};
use tracing::info;

pub struct TestHttpApp {
    pub app_config: Arc<Cfg>,
    pub client: reqwest::Client,
    pub has_dropped: bool,
    handle: ShutdownSdr,

    should_drop_vhost: bool,
    should_drop_db: bool,
}

impl TestHttpApp {
    pub async fn build_server() -> anyhow::Result<Cfg> {
        let mut config = Launcher::get_config(None)?;
        config.main_cfg.port = 0;
        config.main_cfg.run_migration = true;
        info!("modify the config opts");
        Ok(config)
    }

    pub async fn setup(
        mut cfg: Cfg,
        vhost: Option<String>,
        email_client: Option<Box<dyn EmailSender>>,
    ) -> anyhow::Result<Self> {
        let should_drop_vhost = match vhost {
            Some(vhost) => {
                cfg.rabbitmq_cfg.vhost = vhost;
                false
            }
            None => {
                cfg.rabbitmq_cfg.vhost = create_random_vhost(
                    &reqwest::Client::new(),
                    &cfg.rabbitmq_cfg.manage_url().unwrap(),
                )
                .await?;
                true
            }
        };
        let db_name = uuid::Uuid::new_v4().to_string();
        cfg.db_cfg.db = db_name;

        let mut client = reqwest::Client::builder().timeout(Duration::from_secs(2));
        if cfg.main_cfg.tls.is_tls_on()? {
            let pem = tokio::fs::read(cfg.main_cfg.tls.ca_tls_cert_path.as_ref().unwrap()).await?;
            let cert = reqwest::Certificate::from_pem(&pem)?;
            client = client.add_root_certificate(cert)
        }

        let mut app = Launcher::build_from_config(cfg).await?;
        info!("Server is built");
        let handle = app.get_abort_handle();
        app.email_client = email_client;
        let notify = app.started_notify.clone();
        let app_config = app.shared_data.clone();
        info!("starting http server");
        tokio::spawn(async move {
            app.run_forever().await.unwrap();
        });
        info!("Waiting for http server to start");
        notify.notified().await;
        info!("http server started. Build TestHttpApp done");

        let client = client.build()?;
        Ok(TestHttpApp {
            client,
            app_config,
            has_dropped: false,
            handle,
            should_drop_vhost,
            should_drop_db: true,
        })
    }

    pub async fn new(email_client: Option<Box<dyn EmailSender>>) -> anyhow::Result<Self> {
        Self::setup(Self::build_server().await?, None, email_client).await
    }

    pub async fn ourchat_api_get(
        &self,
        name: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_get(format!("v1/{}", name.as_ref())).await
    }

    pub async fn matrix_api_get(
        &self,
        name: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_get(format!("_matrix/{}", name.as_ref())).await
    }

    pub async fn http_get(
        &self,
        url: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let base_url = self.app_config.main_cfg.base_url();
        self.client
            .get(format!("{}{}", base_url, url.as_ref()))
            .send()
            .await
    }

    pub async fn verify(&mut self, token: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.ourchat_api_get(format!("verify/confirm?token={}", token))
            .await
    }

    pub async fn async_drop(&mut self) {
        self.handle.shutdown_all_tasks().await.unwrap();
        if self.should_drop_vhost {
            delete_vhost(
                &reqwest::Client::new(),
                &self.app_config.rabbitmq_cfg.manage_url().unwrap(),
                &self.app_config.rabbitmq_cfg.vhost,
            )
            .await
            .unwrap();
            info!("vhost was deleted");
        }
        if self.should_drop_db {
            match sqlx::Postgres::drop_database(&self.app_config.db_cfg.url()).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("failed to drop database: {}", e);
                }
            }
        }
        self.has_dropped = true;
        info!("async drop done");
    }
}

impl Drop for TestHttpApp {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this app");
        }
    }
}
