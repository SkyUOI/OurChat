use crate::service::{HealthService, Service, ServiceRegistration};
use anyhow::Result;
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub address: String,
    pub datacenter: Option<String>,
    pub token: Option<String>,
    pub timeout: Duration,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self {
            address: "http://127.0.0.1:8500".to_string(),
            datacenter: None,
            token: None,
            timeout: Duration::from_secs(10),
        })
    }

    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = address.into();
        self
    }

    pub fn with_datacenter(mut self, datacenter: impl Into<String>) -> Self {
        self.datacenter = Some(datacenter.into());
        self
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Clone)]
pub struct Client {
    config: Config,
    http_client: HttpClient,
}

impl Client {
    pub fn new(config: Config, address: &str) -> Self {
        let config = config.with_address(address);
        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    pub async fn register(&self, registration: ServiceRegistration) -> Result<()> {
        let url = format!("{}/v1/agent/service/register", self.config.address);
        self.put(&url, &registration).await
    }

    pub async fn deregister(&self, service_id: &str) -> Result<()> {
        let url = format!(
            "{}/v1/agent/service/deregister/{}",
            self.config.address, service_id
        );
        self.put(&url, &()).await
    }

    pub async fn get_service(&self, service_name: &str) -> Result<Vec<Service>> {
        let url = format!(
            "{}/v1/catalog/service/{}",
            self.config.address, service_name
        );
        self.get(&url).await
    }

    pub async fn get_healthy_service(&self, service_name: &str) -> Result<Vec<HealthService>> {
        let url = format!("{}/v1/health/service/{}", self.config.address, service_name);
        self.get(&url).await
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let mut request = self.http_client.get(url);

        if let Some(token) = &self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        if let Some(dc) = &self.config.datacenter {
            request = request.query(&[("dc", dc)]);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Consul request failed: {} - {}", status, text);
        }

        Ok(response.json().await?)
    }

    async fn put<T: serde::Serialize>(&self, url: &str, payload: &T) -> Result<()> {
        let mut request = self.http_client.put(url).json(payload);

        if let Some(token) = &self.config.token {
            request = request.header("X-Consul-Token", token);
        }

        if let Some(dc) = &self.config.datacenter {
            request = request.query(&[("dc", dc)]);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Consul request failed: {} - {}", status, text);
        }

        Ok(())
    }

    // KV Store operations
    pub async fn kv_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let url = format!("{}/v1/kv/{}", self.config.address, key);
        let response = self.get::<Vec<KVPair>>(&url).await?;
        Ok(response.first().map(|pair| pair.value.clone()))
    }

    pub async fn kv_put(
        &self,
        key: &str,
        value: impl std::fmt::Display + serde::Serialize,
    ) -> Result<()> {
        let url = format!("{}/v1/kv/{}", self.config.address, key);
        self.put(&url, &value).await
    }

    pub async fn kv_delete(&self, key: &str) -> Result<()> {
        let url = format!("{}/v1/kv/{}", self.config.address, key);
        let response = self.http_client.delete(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            anyhow::bail!("Consul KV delete failed: {} - {}", status, text);
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct KVPair {
    #[serde(rename = "Value")]
    value: Vec<u8>,
}
