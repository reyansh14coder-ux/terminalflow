#![allow(dead_code)]

use anyhow::Result;
use std::collections::HashMap;

pub trait Middleware: Send + Sync {
    fn before_request(&self, _request: &mut super::client::HttpRequest) -> Result<()> {
        Ok(())
    }

    fn after_response(&self, _response: &mut super::client::HttpResponse) -> Result<()> {
        Ok(())
    }
}

pub struct LoggingMiddleware {
    verbose: bool,
}

impl LoggingMiddleware {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl Middleware for LoggingMiddleware {
    fn before_request(&self, request: &mut super::client::HttpRequest) -> Result<()> {
        if self.verbose {
            println!("→ {} {}", request.method, request.url);
            for (key, value) in &request.headers {
                println!("  {}: {}", key, value);
            }
        }
        Ok(())
    }

    fn after_response(&self, response: &mut super::client::HttpResponse) -> Result<()> {
        if self.verbose {
            println!(
                "← {} {} ({}ms)",
                response.status, response.status_text, response.duration_ms
            );
        }
        Ok(())
    }
}

pub struct AuthMiddleware {
    token: String,
}

impl AuthMiddleware {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

impl Middleware for AuthMiddleware {
    fn before_request(&self, request: &mut super::client::HttpRequest) -> Result<()> {
        request.headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.token),
        );
        Ok(())
    }
}

pub struct RetryMiddleware {
    max_retries: u32,
    delay_ms: u64,
}

impl RetryMiddleware {
    pub fn new(max_retries: u32, delay_ms: u64) -> Self {
        Self {
            max_retries,
            delay_ms,
        }
    }
}

impl Middleware for RetryMiddleware {
    fn after_response(&self, _response: &mut super::client::HttpResponse) -> Result<()> {
        // In real implementation, this would retry on failure
        Ok(())
    }
}

pub struct CacheMiddleware {
    cache: HashMap<String, CachedResponse>,
    ttl_ms: u64,
}

struct CachedResponse {
    response: super::client::HttpResponse,
    timestamp: std::time::Instant,
}

impl CacheMiddleware {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl_ms: ttl_seconds * 1000,
        }
    }
}

impl Middleware for CacheMiddleware {
    fn before_request(&self, _request: &mut super::client::HttpRequest) -> Result<()> {
        // Check cache
        Ok(())
    }

    fn after_response(&self, _response: &mut super::client::HttpResponse) -> Result<()> {
        // Store in cache
        Ok(())
    }
}
