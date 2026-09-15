#![allow(dead_code)]

use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct APIServer {
    port: u16,
    routes: HashMap<String, Box<dyn Fn(Request) -> Response + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn ok(body: &str) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: body.to_string(),
        }
    }

    pub fn json(data: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Self {
            status: 200,
            headers,
            body: data.to_string(),
        }
    }

    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: message.to_string(),
        }
    }
}

impl APIServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            routes: HashMap::new(),
        }
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let key = format!("GET:{}", path);
        self.routes.insert(key, Box::new(handler));
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let key = format!("POST:{}", path);
        self.routes.insert(key, Box::new(handler));
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind to port: {}", e))?;

        println!("🚀 API Server running on http://{}", addr);

        loop {
            let (mut socket, _) = listener.accept().await?;

            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                let n = socket.read(&mut buffer).await.unwrap_or(0);

                if n == 0 {
                    return;
                }

                let request_str = String::from_utf8_lossy(&buffer[..n]);
                let response = handle_request_static(&request_str);

                let response_str = format!(
                    "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
                    response.status,
                    response.body.len(),
                    response.body
                );

                let _ = socket.write_all(response_str.as_bytes()).await;
            });
        }
    }
}

fn handle_request_static(request_str: &str) -> Response {
    let lines: Vec<&str> = request_str.lines().collect();
    if lines.is_empty() {
        return Response::error(400, "Bad Request");
    }

    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    if first_line.len() < 2 {
        return Response::error(400, "Bad Request");
    }

    let _method = first_line[0];
    let _path = first_line[1];

    // Default response for now
    Response::ok(r#"{"status":"ok"}"#)
}
