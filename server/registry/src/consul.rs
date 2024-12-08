use anyhow::Result;
use consul_client::{Client, Service, ServiceRegistration, client::Config};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub metadata: HashMap<String, String>,
}

pub struct ConsulRegistry {
    client: Client,
    service_name: String,
    instance_id: String,
    address: IpAddr,
    port: u16,
}

impl ConsulRegistry {
    pub fn new(consul_addr: &str, service_name: &str, address: IpAddr, port: u16) -> Result<Self> {
        let config = Config::new().map_err(anyhow::Error::msg)?;
        let client = Client::new(config, consul_addr);
        let instance_id = format!("{}-{}", service_name, Uuid::new_v4());

        Ok(Self {
            client,
            service_name: service_name.to_string(),
            instance_id,
            address,
            port,
        })
    }

    // Register service instance
    pub async fn register_service(&self) -> Result<()> {
        let registration = ServiceRegistration::new(
            self.service_name.clone(),
            self.instance_id.clone(),
            self.address,
            self.port,
        )
        .add_tag("chat-service");

        self.client.register(registration).await?;
        Ok(())
    }

    // Deregister service instance
    pub async fn deregister_service(&self) -> Result<()> {
        self.client.deregister(&self.instance_id).await?;
        Ok(())
    }

    // Register a new connection
    pub async fn register_connection(
        &self,
        connection_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let connection_info = ConnectionInfo {
            connection_id: connection_id.to_string(),
            metadata,
        };

        // Add the connection info to metadata
        let key = format!(
            "service/{}/{}/connections/{}",
            self.service_name, self.instance_id, connection_id
        );
        let value = serde_json::to_string(&connection_info)?;

        self.client.kv_put(&key, value).await?;
        Ok(())
    }

    // Find a connection by connection_id and return the service instance address
    pub async fn find_connection(
        &self,
        connection_id: &str,
    ) -> Result<Option<(ConnectionInfo, SocketAddr)>> {
        // Get all healthy service instances
        let services = self.client.get_healthy_service(&self.service_name).await?;

        for service in services {
            if !service.health_checks.iter().all(|check| check.is_passing()) {
                continue;
            }

            let service = service.service;
            let key = format!(
                "service/{}/{}/connections/{}",
                self.service_name, service.id, connection_id
            );

            if let Some(value) = self.client.kv_get(&key).await? {
                let connection_info: ConnectionInfo = serde_json::from_slice(&value)?;
                let addr = SocketAddr::new(service.address, service.port);
                return Ok(Some((connection_info, addr)));
            }
        }

        Ok(None)
    }

    // Deregister a connection
    pub async fn deregister_connection(&self, connection_id: &str) -> Result<()> {
        let key = format!(
            "service/{}/{}/connections/{}",
            self.service_name, self.instance_id, connection_id
        );
        self.client.kv_delete(&key).await?;
        Ok(())
    }

    // Get all healthy service instances
    pub async fn get_healthy_instances(&self) -> Result<Vec<Service>> {
        let services = self.client.get_healthy_service(&self.service_name).await?;

        Ok(services
            .into_iter()
            .filter(|service| service.health_checks.iter().all(|check| check.is_passing()))
            .map(|service| service.service)
            .collect())
    }
}
