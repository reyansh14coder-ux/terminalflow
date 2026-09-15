#![allow(dead_code)]

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout: Option<u64>,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
}

pub struct APIClient {
    client: Client,
    base_url: Option<String>,
    default_headers: HashMap<String, String>,
}

impl APIClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            client,
            base_url: None,
            default_headers: HashMap::new(),
        })
    }

    pub fn with_base_url(base_url: &str) -> Result<Self> {
        let mut client = Self::new()?;
        client.base_url = Some(base_url.to_string());
        Ok(client)
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.default_headers.insert(key.to_string(), value.to_string());
    }

    pub async fn get(&self, path: &str) -> Result<APIResponse> {
        self.execute(&APIRequest {
            method: "GET".to_string(),
            url: path.to_string(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }).await
    }

    pub async fn post(&self, path: &str, body: Option<String>) -> Result<APIResponse> {
        self.execute(&APIRequest {
            method: "POST".to_string(),
            url: path.to_string(),
            headers: HashMap::new(),
            body,
            timeout: None,
            follow_redirects: true,
        }).await
    }

    pub async fn execute(&self, request: &APIRequest) -> Result<APIResponse> {
        let url = if request.url.starts_with("http") {
            request.url.clone()
        } else {
            match &self.base_url {
                Some(base) => format!("{}{}", base, request.url),
                None => request.url.clone(),
            }
        };
        
        let start = std::time::Instant::now();
        
        let mut req = match request.method.as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            "PATCH" => self.client.patch(&url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", request.method)),
        };
        
        for (key, value) in &request.headers {
            req = req.header(key.as_str(), value.as_str());
        }
        
        for (key, value) in &self.default_headers {
            req = req.header(key.as_str(), value.as_str());
        }
        
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }
        
        let response = req.send().await.context("Request failed")?;
        let duration = start.elapsed().as_millis() as u64;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await.context("Failed to read response body")?;
        
        Ok(APIResponse {
            status,
            headers,
            body,
            duration_ms: duration,
        })
    }
}
