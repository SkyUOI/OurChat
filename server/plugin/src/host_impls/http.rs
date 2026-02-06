//! HTTP client host implementation
//!
//! Provides HTTP client functionality to plugins via the WIT HTTP interface.

use crate::engine::PluginState;
use parking_lot::RwLock;
use reqwest::Client as ReqwestClient;
use std::sync::Arc;
use std::time::Duration;

/// HTTP request methods
#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// HTTP request headers
pub type Headers = Vec<(String, String)>;

/// HTTP request
#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Headers,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u32>,
}

/// HTTP response
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Headers,
    pub body: Vec<u8>,
}

/// HTTP error
#[derive(Debug)]
pub enum HttpError {
    NetworkError(String),
    Timeout,
    InvalidUrl,
    ConnectionFailed,
}

/// Security settings for HTTP access
struct HttpSecurity {
    /// Blocked network ranges (for internal network access prevention)
    blocked_ranges: Vec<String>,
    /// Allowed domains (if set, only these domains are allowed)
    allowed_domains: Option<Vec<String>>,
}

impl HttpSecurity {
    fn new() -> Self {
        Self {
            // Block internal network ranges
            blocked_ranges: vec![
                "127.0.0.0/8".to_string(),
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
            allowed_domains: None, // TODO: Load from config
        }
    }

    /// Validate URL to prevent internal network access
    fn validate_url(&self, url: &str) -> Result<(), HttpError> {
        // Parse URL
        let parsed = url::Url::parse(url).map_err(|_| HttpError::InvalidUrl)?;

        // Check if hostname is in blocked ranges
        let host = parsed.host_str().ok_or(HttpError::InvalidUrl)?;

        // Block localhost and internal IPs
        if host == "localhost" || host.starts_with("127.") || host.starts_with("10.") ||
            host.starts_with("192.168.") || (host.starts_with("172.") &&
            host[4..6].parse::<u8>().map_or(false, |n| (16..=31).contains(&n))) {
            return Err(HttpError::ConnectionFailed);
        }

        // Check allowed domains if configured
        if let Some(allowed) = &self.allowed_domains {
            if !allowed.iter().any(|d| host.ends_with(d)) {
                return Err(HttpError::ConnectionFailed);
            }
        }

        Ok(())
    }
}

/// Host implementation for the HTTP interface
pub struct HttpHost {
    client: ReqwestClient,
    security: HttpSecurity,
    state: Arc<RwLock<PluginState>>,
}

impl HttpHost {
    pub fn new(state: Arc<RwLock<PluginState>>) -> Self {
        let client = ReqwestClient::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            security: HttpSecurity::new(),
            state,
        }
    }

    fn convert_method(method: &HttpMethod) -> reqwest::Method {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
        }
    }
}

/// Implement the HTTP interface from WIT
impl HttpHost {
    pub async fn send(&self, req: HttpRequest) -> Result<HttpResponse, HttpError> {
        // Validate URL for security
        self.security.validate_url(&req.url)?;

        let state = self.state.read();
        tracing::debug!(
            plugin = %state.plugin_id,
            "HTTP request: {:?} {}",
            req.method, req.url
        );

        // Build request
        let mut request = self.client.request(
            Self::convert_method(&req.method),
            &req.url
        );

        // Add headers
        for (key, value) in req.headers {
            request = request.header(&key, &value);
        }

        // Add body
        if let Some(body) = req.body {
            request = request.body(body);
        }

        // Set timeout
        if let Some(timeout_ms) = req.timeout_ms {
            request = request.timeout(Duration::from_millis(timeout_ms as u64));
        }

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| HttpError::NetworkError(e.to_string()))?;

        // Collect response
        let status_code = response.status().as_u16();
        let headers: Headers = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| HttpError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }

    pub async fn get(&self, url: String) -> Result<HttpResponse, HttpError> {
        self.send(HttpRequest {
            method: HttpMethod::Get,
            url,
            headers: Vec::new(),
            body: None,
            timeout_ms: None,
        })
        .await
    }

    pub async fn post_json(&self, url: String, body: String) -> Result<HttpResponse, HttpError> {
        let mut headers = Vec::new();
        headers.push(("Content-Type".to_string(), "application/json".to_string()));

        self.send(HttpRequest {
            method: HttpMethod::Post,
            url,
            headers,
            body: Some(body.into_bytes()),
            timeout_ms: None,
        })
        .await
    }
}
