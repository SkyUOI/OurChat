use base::email_client::EmailSender;
use http_server::Launcher;
use std::{thread, time::Duration};
use tokio::task::JoinHandle;

use crate::helper::rabbitmq::create_random_vhost;

pub struct TestHttpApp {
    pub app_config: http_server::Config,
    pub client: reqwest::Client,
    pub has_dropped: bool,
    handle: JoinHandle<()>,

    should_drop_vhost: bool,
}

impl TestHttpApp {
    pub async fn build_server(
        email_client: Option<Box<dyn EmailSender>>,
    ) -> anyhow::Result<Launcher> {
        let mut config = Launcher::get_config(None)?;
        config.port = 0;
        let mut app = Launcher::build_from_config(config).await?;
        app.email_client = email_client;
        tracing::info!("build server and modify the config opts");
        Ok(app)
    }

    pub async fn setup(mut app: Launcher, vhost: Option<String>) -> anyhow::Result<Self> {
        let app_config = app.config.clone();
        let notify = app.started_notify.clone();
        let should_drop_vhost = match vhost {
            Some(vhost) => {
                app.rabbitmq_cfg.vhost = vhost;
                false
            }
            None => {
                app.rabbitmq_cfg.vhost = create_random_vhost(
                    &reqwest::Client::new(),
                    &app.rabbitmq_cfg.manage_url().unwrap(),
                )
                .await?;
                true
            }
        };
        tracing::info!("starting http server");
        let handle = tokio::spawn(async move {
            app.run_forever().await.unwrap();
        });
        tracing::info!("Waiting for http server to start");
        notify.notified().await;
        tracing::info!("http server started. Build TestHttpApp done");
        Ok(TestHttpApp {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
            app_config,
            has_dropped: false,
            handle,
            should_drop_vhost,
        })
    }

    pub async fn new(email_client: Option<Box<dyn EmailSender>>) -> anyhow::Result<Self> {
        Self::setup(Self::build_server(email_client).await?, None).await
    }

    pub async fn http_get(
        &self,
        name: impl AsRef<str>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .get(format!(
                "{}://{}:{}/v1/{}",
                self.app_config.protocol_http(),
                self.app_config.ip,
                self.app_config.port,
                name.as_ref()
            ))
            .send()
            .await
    }

    pub async fn verify(&mut self, token: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.http_get(format!("verify/confirm?token={}", token))
            .await
    }

    pub async fn async_drop(&mut self) {
        self.handle.abort();
        self.has_dropped = true;
    }
}

impl Drop for TestHttpApp {
    fn drop(&mut self) {
        if !self.has_dropped && !thread::panicking() {
            panic!("async_drop is not called to drop this app");
        }
    }
}
