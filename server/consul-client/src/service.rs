use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
    pub tags: Vec<String>,
    #[serde(default)]
    pub meta: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceRegistration {
    pub name: String,
    pub id: String,
    pub address: IpAddr,
    pub port: u16,
    pub tags: Vec<String>,
    #[serde(default)]
    pub meta: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    #[serde(rename = "DeregisterCriticalServiceAfter")]
    pub deregister_critical_service_after: String,
    #[serde(rename = "HTTP")]
    pub http: Option<String>,
    #[serde(rename = "Interval")]
    pub interval: String,
    #[serde(rename = "Timeout")]
    pub timeout: String,
}

impl ServiceRegistration {
    pub fn new(name: String, id: String, address: IpAddr, port: u16) -> Self {
        Self {
            name,
            id,
            address,
            port,
            tags: Vec::new(),
            meta: HashMap::new(),
            check: None,
        }
    }

    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn add_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }

    pub fn with_http_health_check(mut self, endpoint: impl Into<String>) -> Self {
        self.check = Some(HealthCheck {
            deregister_critical_service_after: "30s".to_string(),
            http: Some(endpoint.into()),
            interval: "10s".to_string(),
            timeout: "5s".to_string(),
        });
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthService {
    pub service: Service,
    #[serde(rename = "Checks")]
    pub health_checks: Vec<HealthState>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthState {
    #[serde(rename = "Status")]
    pub status: String,
}

impl HealthState {
    pub fn is_passing(&self) -> bool {
        self.status == "passing"
    }
}
