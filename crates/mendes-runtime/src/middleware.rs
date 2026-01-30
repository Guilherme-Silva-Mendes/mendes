//! Mendes middleware system

use crate::error::Result;
use crate::http::{Request, Response};
use async_trait::async_trait;

/// Trait for middlewares
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Executed before the handler
    /// Returns Ok(Some(response)) to short-circuit
    /// Returns Ok(None) to continue
    async fn before(&self, req: &Request) -> Result<Option<Response>> {
        let _ = req;
        Ok(None)
    }

    /// Executed after the handler
    async fn after(&self, req: &Request, resp: Response) -> Response {
        let _ = req;
        resp
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    /// Expected header
    pub header: String,
    /// Expected prefix (e.g., "Bearer ")
    pub prefix: Option<String>,
}

impl AuthMiddleware {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            prefix: None,
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn bearer() -> Self {
        Self::new("authorization").with_prefix("Bearer ")
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, req: &Request) -> Result<Option<Response>> {
        match req.header(&self.header) {
            Some(value) => {
                if let Some(prefix) = &self.prefix {
                    if !value.starts_with(prefix) {
                        return Ok(Some(Response::unauthorized("Invalid authorization format")));
                    }
                }
                Ok(None)
            }
            None => Ok(Some(Response::unauthorized("Missing authorization header"))),
        }
    }
}

/// Logging middleware
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, req: &Request) -> Result<Option<Response>> {
        tracing::info!("{} {}", req.method, req.path);
        Ok(None)
    }

    async fn after(&self, req: &Request, resp: Response) -> Response {
        tracing::info!("{} {} -> {}", req.method, req.path, resp.status);
        resp
    }
}

/// CORS middleware
pub struct CorsMiddleware {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

impl CorsMiddleware {
    pub fn permissive() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
        }
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::permissive()
    }
}

#[async_trait]
impl Middleware for CorsMiddleware {
    async fn after(&self, _req: &Request, resp: Response) -> Response {
        resp.with_header("Access-Control-Allow-Origin", self.allowed_origins.join(", "))
            .with_header("Access-Control-Allow-Methods", self.allowed_methods.join(", "))
            .with_header("Access-Control-Allow-Headers", self.allowed_headers.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_auth_middleware_missing_header() {
        let middleware = AuthMiddleware::new("authorization");
        let req = Request::new("GET", "/test");

        let result = middleware.before(&req).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, 401);
    }

    #[tokio::test]
    async fn test_auth_middleware_valid() {
        let middleware = AuthMiddleware::new("authorization");
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "token123".to_string());
        let req = Request::new("GET", "/test").with_headers(headers);

        let result = middleware.before(&req).await.unwrap();
        assert!(result.is_none()); // Passed
    }

    #[tokio::test]
    async fn test_cors_middleware() {
        let middleware = CorsMiddleware::permissive();
        let req = Request::new("GET", "/test");
        let resp = Response::ok("test");

        let resp = middleware.after(&req, resp).await;
        assert!(resp.headers.contains_key("access-control-allow-origin"));
    }
}
