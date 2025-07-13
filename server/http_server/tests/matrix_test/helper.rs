use base::email_client::EmailSender;
use client::TestHttpApp;
use http_server::Cfg;

pub async fn build_server() -> anyhow::Result<Cfg> {
    let mut ret = TestHttpApp::get_config().await?;
    ret.main_cfg.enable_matrix = true;
    Ok(ret)
}

pub async fn new(email_client: Option<Box<dyn EmailSender>>) -> anyhow::Result<TestHttpApp> {
    TestHttpApp::setup(build_server().await?, None, email_client).await
}
