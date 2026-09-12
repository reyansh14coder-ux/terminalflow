use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout: Option<u64>,
    pub follow_redirects: bool,
}

impl Default for HttpRequest {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            url: String::new(),
            headers: HashMap::new(),
            query: HashMap::new(),
            body: None,
            timeout: Some(30),
            follow_redirects: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
    pub size_bytes: usize,
    pub url: String,
}

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self { client })
    }

    pub async fn execute(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let start = std::time::Instant::now();
        
        let mut req = match request.method.as_str() {
            "GET" => self.client.get(&request.url),
            "POST" => self.client.post(&request.url),
            "PUT" => self.client.put(&request.url),
            "DELETE" => self.client.delete(&request.url),
            "PATCH" => self.client.patch(&request.url),
            "HEAD" => self.client.head(&request.url),
            _ => return Err(anyhow::anyhow!("Unsupported method: {}", request.method)),
        };
        
        for (key, value) in &request.headers {
            req = req.header(key.as_str(), value.as_str());
        }
        
        for (key, value) in &request.query {
            req = req.query(&[(key.as_str(), value.as_str())]);
        }
        
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }
        
        let response = req.send().await.context("Request failed")?;
        let duration = start.elapsed().as_millis() as u64;
        
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
        let url = response.url().to_string();
        
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await.context("Failed to read body")?;
        let size_bytes = body.len();
        
        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
            duration_ms: duration,
            size_bytes,
            url,
        })
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        let request = HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            ..Default::default()
        };
        self.execute(&request).await
    }

    pub async fn post_json(&self, url: &str, json: &str) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        let request = HttpRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers,
            body: Some(json.to_string()),
            ..Default::default()
        };
        self.execute(&request).await
    }
}
