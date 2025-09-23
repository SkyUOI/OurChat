use client::{TestApp, oc_helper::client::ConfigWithArgs};

pub fn get_test_config() -> anyhow::Result<ConfigWithArgs> {
    let mut ret = TestApp::get_test_config()?;
    ret.0.http_cfg.enable_matrix = true;
    Ok(ret)
}

pub async fn new_with_launching_instance() -> anyhow::Result<TestApp> {
    TestApp::new_with_launching_instance_custom_cfg(get_test_config()?, |_| {}).await
}
