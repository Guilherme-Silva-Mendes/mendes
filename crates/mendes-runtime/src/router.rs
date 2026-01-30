//! Mendes HTTP Router
//!
//! Routing system with support for:
//! - Path parameters (/users/{id})
//! - Per-route middlewares
//! - HTTP methods (GET, POST, PUT, DELETE, PATCH)

use crate::http::{Request, Response};
use crate::middleware::Middleware;
use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Async handler type
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for HTTP handlers
#[async_trait]
pub trait Handler: Send + Sync {
    async fn call(&self, req: Request) -> Response;
}

/// Handler implementation for closures
pub struct FnHandler<F>(pub F);

#[async_trait]
impl<F, Fut> Handler for FnHandler<F>
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future<Output = Response> + Send,
{
    async fn call(&self, req: Request) -> Response {
        (self.0)(req).await
    }
}

/// HTTP Route
struct Route {
    method: String,
    pattern: RoutePattern,
    handler: Arc<dyn Handler>,
    middlewares: Vec<Arc<dyn Middleware>>,
}

/// Route pattern (with parameter support)
#[derive(Debug, Clone)]
struct RoutePattern {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
enum Segment {
    /// Literal segment
    Literal(String),
    /// Path parameter {name} or {name:type}
    Param { name: String, param_type: Option<String> },
}

impl RoutePattern {
    fn parse(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.starts_with('{') && s.ends_with('}') {
                    let inner = &s[1..s.len() - 1];
                    let (name, param_type) = if let Some(idx) = inner.find(':') {
                        (inner[..idx].to_string(), Some(inner[idx + 1..].to_string()))
                    } else {
                        (inner.to_string(), None)
                    };
                    Segment::Param { name, param_type }
                } else {
                    Segment::Literal(s.to_string())
                }
            })
            .collect();

        Self { segments }
    }

    /// Tries to match a path and extracts parameters
    fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if path_segments.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_seg, path_seg) in self.segments.iter().zip(path_segments.iter()) {
            match pattern_seg {
                Segment::Literal(lit) => {
                    if lit != *path_seg {
                        return None;
                    }
                }
                Segment::Param { name, param_type } => {
                    // Basic type validation
                    if let Some(ty) = param_type {
                        match ty.as_str() {
                            "int" => {
                                if path_seg.parse::<i64>().is_err() {
                                    return None;
                                }
                            }
                            _ => {}
                        }
                    }
                    params.insert(name.clone(), path_seg.to_string());
                }
            }
        }

        Some(params)
    }
}

/// WebSocket route
struct WsRoute {
    pattern: RoutePattern,
    handler: Arc<dyn Fn(crate::WsConnection) -> BoxFuture<'static, ()> + Send + Sync>,
}

/// HTTP Router
pub struct Router {
    routes: Vec<Route>,
    ws_routes: Vec<WsRoute>,
    global_middlewares: Vec<Arc<dyn Middleware>>,
}

impl Router {
    /// Creates new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            ws_routes: Vec::new(),
            global_middlewares: Vec::new(),
        }
    }

    /// Adds global middleware
    pub fn middleware<M: Middleware + 'static>(&mut self, middleware: M) -> &mut Self {
        self.global_middlewares.push(Arc::new(middleware));
        self
    }

    /// Registers GET route
    pub fn get<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route_internal("GET", path, handler)
    }

    /// Registers POST route
    pub fn post<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route_internal("POST", path, handler)
    }

    /// Registers PUT route
    pub fn put<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route_internal("PUT", path, handler)
    }

    /// Registers DELETE route
    pub fn delete<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route_internal("DELETE", path, handler)
    }

    /// Registers PATCH route
    pub fn patch<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.add_route_internal("PATCH", path, handler)
    }

    /// Registers WebSocket route
    pub fn ws<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(crate::WsConnection) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let ws_route = WsRoute {
            pattern: RoutePattern::parse(path),
            handler: Arc::new(move |conn| {
                let fut = handler(conn);
                Box::pin(fut) as BoxFuture<'static, ()>
            }),
        };
        self.ws_routes.push(ws_route);
        self
    }

    /// Returns the WebSocket routes
    pub fn ws_routes(&self) -> &[WsRoute] {
        &self.ws_routes
    }

    /// Checks if a path matches a WebSocket route and returns the handler
    pub fn match_ws(&self, path: &str) -> Option<&Arc<dyn Fn(crate::WsConnection) -> BoxFuture<'static, ()> + Send + Sync>> {
        for ws_route in &self.ws_routes {
            if let Some(_params) = ws_route.pattern.match_path(path) {
                return Some(&ws_route.handler);
            }
        }
        None
    }

    /// Registers generic route
    fn add_route_internal<F, Fut>(&mut self, method: &str, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let route = Route {
            method: method.to_string(),
            pattern: RoutePattern::parse(path),
            handler: Arc::new(FnHandler(handler)),
            middlewares: Vec::new(),
        };
        self.routes.push(route);
        self
    }

    /// Registers handler directly (used by generated code)
    pub fn add_handler(&mut self, method: &str, path: &str, handler: Arc<dyn Handler>) {
        let route = Route {
            method: method.to_string(),
            pattern: RoutePattern::parse(path),
            handler,
            middlewares: Vec::new(),
        };
        self.routes.push(route);
    }

    /// Routes request and returns response
    pub async fn handle(&self, req: &Request) -> Option<Response> {
        for route in &self.routes {
            if route.method != req.method {
                continue;
            }

            if let Some(params) = route.pattern.match_path(&req.path) {
                let mut request = req.clone().with_params(params);

                // Execute global middlewares
                for middleware in &self.global_middlewares {
                    match middleware.before(&request).await {
                        Ok(Some(resp)) => return Some(resp), // Middleware returned response
                        Ok(None) => {} // Continue
                        Err(e) => return Some(Response::error(500, e.to_string())),
                    }
                }

                // Execute route middlewares
                for middleware in &route.middlewares {
                    match middleware.before(&request).await {
                        Ok(Some(resp)) => return Some(resp),
                        Ok(None) => {}
                        Err(e) => return Some(Response::error(500, e.to_string())),
                    }
                }

                // Execute handler
                let mut response = route.handler.call(request.clone()).await;

                // Execute after middlewares (in reverse order)
                for middleware in self.global_middlewares.iter().rev() {
                    response = middleware.after(&request, response).await;
                }

                return Some(response);
            }
        }

        None
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_pattern_literal() {
        let pattern = RoutePattern::parse("/users/list");
        let params = pattern.match_path("/users/list");
        assert!(params.is_some());
        assert!(params.unwrap().is_empty());

        assert!(pattern.match_path("/users/other").is_none());
    }

    #[test]
    fn test_route_pattern_param() {
        let pattern = RoutePattern::parse("/users/{id}");
        let params = pattern.match_path("/users/123");
        assert!(params.is_some());
        let params = params.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_route_pattern_typed_param() {
        let pattern = RoutePattern::parse("/users/{id:int}");

        // Valid: number
        let params = pattern.match_path("/users/123");
        assert!(params.is_some());

        // Invalid: not a number
        let params = pattern.match_path("/users/abc");
        assert!(params.is_none());
    }

    #[test]
    fn test_route_pattern_multiple_params() {
        let pattern = RoutePattern::parse("/users/{user_id}/posts/{post_id}");
        let params = pattern.match_path("/users/1/posts/42");
        assert!(params.is_some());
        let params = params.unwrap();
        assert_eq!(params.get("user_id"), Some(&"1".to_string()));
        assert_eq!(params.get("post_id"), Some(&"42".to_string()));
    }
}
