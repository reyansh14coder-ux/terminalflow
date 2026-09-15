#![allow(dead_code)]

use std::collections::HashMap;
use anyhow::Result;

type Handler = Box<dyn Fn(Request) -> Response + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Request {
    pub path: String,
    pub method: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
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

    pub fn not_found() -> Self {
        Self {
            status: 404,
            headers: HashMap::new(),
            body: "Not Found".to_string(),
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

pub struct Router {
    routes: Vec<Route>,
    middleware: Vec<Box<dyn Middleware>>,
}

struct Route {
    method: String,
    path: String,
    handler: Handler,
}

trait Middleware: Send + Sync {
    fn before(&self, _request: &mut Request) -> Result<()> {
        Ok(())
    }
    fn after(&self, _response: &mut Response) -> Result<()> {
        Ok(())
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.routes.push(Route {
            method: "GET".to_string(),
            path: path.to_string(),
            handler: Box::new(handler),
        });
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.routes.push(Route {
            method: "POST".to_string(),
            path: path.to_string(),
            handler: Box::new(handler),
        });
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.routes.push(Route {
            method: "PUT".to_string(),
            path: path.to_string(),
            handler: Box::new(handler),
        });
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.routes.push(Route {
            method: "DELETE".to_string(),
            path: path.to_string(),
            handler: Box::new(handler),
        });
    }

    pub fn route(&self, request: &Request) -> Response {
        for route in &self.routes {
            if route.method == request.method && self.path_matches(&route.path, &request.path) {
                return (route.handler)(request.clone());
            }
        }
        Response::not_found()
    }

    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with(':') {
                continue;
            }
            if pattern_part != path_part {
                return false;
            }
        }

        true
    }
}
