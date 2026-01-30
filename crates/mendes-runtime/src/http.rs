//! Mendes HTTP Server
//!
//! High-performance asynchronous HTTP server based on Hyper.

use crate::error::{HttpError, MendesError, Result};
use crate::router::Router;
use crate::types::MendesString;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub use hyper::StatusCode;

/// Mendes HTTP Request
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// Path parameters (extracted from route)
    pub params: HashMap<String, String>,
    /// Request body
    body: Vec<u8>,
}

impl Request {
    /// Creates request for testing
    #[cfg(test)]
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            headers: HashMap::new(),
            query: HashMap::new(),
            params: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Creates request with headers (for testing)
    #[cfg(test)]
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Creates request from hyper request
    pub(crate) async fn from_hyper(req: hyper::Request<Incoming>) -> Result<Self> {
        let method = req.method().to_string();
        let uri = req.uri();
        let path = uri.path().to_string();

        // Parse query string
        let query: HashMap<String, String> = uri
            .query()
            .map(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_default();

        // Collect headers
        let headers: HashMap<String, String> = req
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // Read body
        let body = req.collect().await
            .map_err(|e| MendesError::Http(HttpError::new(400, e.to_string())))?
            .to_bytes()
            .to_vec();

        Ok(Self {
            method,
            path,
            headers,
            query,
            params: HashMap::new(),
            body,
        })
    }

    /// Gets header
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// Gets query parameter
    pub fn query_param(&self, name: &str) -> Option<&String> {
        self.query.get(name)
    }

    /// Gets path parameter
    pub fn param(&self, name: &str) -> Option<&String> {
        self.params.get(name)
    }

    /// Gets path parameter as int
    pub fn param_int(&self, name: &str) -> Option<i64> {
        self.params.get(name)?.parse().ok()
    }

    /// Gets body as string
    pub fn body_string(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| MendesError::Serialization(e.to_string()))
    }

    /// Gets body as MendesString
    pub fn body_mendes_string(&self) -> Result<MendesString> {
        Ok(MendesString::new(self.body_string()?))
    }

    /// Deserializes body as JSON
    pub fn body_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(MendesError::from)
    }

    /// Sets path parameters (used internally by router)
    pub(crate) fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }
}

/// Mendes HTTP Response
#[derive(Debug, Clone)]
pub struct Response {
    /// Status code
    pub status: u16,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body
    pub body: Vec<u8>,
}

impl Response {
    /// Creates response with status and body
    pub fn new(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Response 200 OK with body
    pub fn ok(body: impl Into<String>) -> Self {
        let body_str = body.into();
        let mut resp = Self::new(200, body_str.into_bytes());
        resp.headers.insert("content-type".to_string(), "text/plain; charset=utf-8".to_string());
        resp
    }

    /// Response 200 OK with JSON
    pub fn json<T: Serialize>(data: &T) -> Result<Self> {
        let json = serde_json::to_string(data)?;
        let mut resp = Self::new(200, json.into_bytes());
        resp.headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(resp)
    }

    /// Error response
    pub fn error(status: u16, message: impl Into<String>) -> Self {
        let msg = message.into();
        let json = format!(r#"{{"error":"{}"}}"#, msg);
        let mut resp = Self::new(status, json.into_bytes());
        resp.headers.insert("content-type".to_string(), "application/json".to_string());
        resp
    }

    /// Response 400 Bad Request
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::error(400, message)
    }

    /// Response 401 Unauthorized
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::error(401, message)
    }

    /// Response 404 Not Found
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::error(404, message)
    }

    /// Response 500 Internal Server Error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::error(500, message)
    }

    /// Adds header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into().to_lowercase(), value.into());
        self
    }

    /// Converts to hyper response
    pub(crate) fn into_hyper(self) -> hyper::Response<Full<Bytes>> {
        let mut builder = hyper::Response::builder().status(self.status);

        for (name, value) in &self.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }

        builder
            .body(Full::new(Bytes::from(self.body)))
            .unwrap_or_else(|_| {
                hyper::Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from("Internal Server Error")))
                    .unwrap()
            })
    }
}

impl From<HttpError> for Response {
    fn from(e: HttpError) -> Self {
        Response::error(e.status, e.message)
    }
}

impl From<MendesString> for Response {
    fn from(s: MendesString) -> Self {
        Response::ok(s.0)
    }
}

impl From<&str> for Response {
    fn from(s: &str) -> Self {
        Response::ok(s)
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        Response::ok(s)
    }
}

impl From<i64> for Response {
    fn from(n: i64) -> Self {
        Response::ok(n.to_string())
    }
}

impl From<f64> for Response {
    fn from(n: f64) -> Self {
        Response::ok(n.to_string())
    }
}

impl From<bool> for Response {
    fn from(b: bool) -> Self {
        Response::ok(if b { "true" } else { "false" })
    }
}

/// Mendes HTTP Server
pub struct Server {
    addr: String,
    router: Option<Router>,
}

impl Server {
    /// Creates new server
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            router: None,
        }
    }

    /// Sets router
    pub fn router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// Starts the server
    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = self.addr.parse()
            .map_err(|e| MendesError::Internal(format!("Invalid address: {}", e)))?;

        let router = Arc::new(self.router.unwrap_or_else(Router::new));

        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Mendes server listening on http://{}", addr);

        loop {
            let (stream, remote_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let router = router.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let router = router.clone();
                    async move {
                        let response = handle_request(req, router).await;
                        Ok::<_, Infallible>(response)
                    }
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .await
                {
                    tracing::error!("Error serving connection from {}: {:?}", remote_addr, err);
                }
            });
        }
    }
}

/// Request handler
async fn handle_request(
    req: hyper::Request<Incoming>,
    router: Arc<Router>,
) -> hyper::Response<Full<Bytes>> {
    // Parse request
    let request = match Request::from_hyper(req).await {
        Ok(r) => r,
        Err(e) => {
            return Response::bad_request(e.to_string()).into_hyper();
        }
    };

    // Route request
    match router.handle(&request).await {
        Some(response) => response.into_hyper(),
        None => Response::not_found("Not Found").into_hyper(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_ok() {
        let resp = Response::ok("hello");
        assert_eq!(resp.status, 200);
        assert_eq!(String::from_utf8(resp.body).unwrap(), "hello");
    }

    #[test]
    fn test_response_error() {
        let resp = Response::error(404, "not found");
        assert_eq!(resp.status, 404);
        assert!(String::from_utf8(resp.body).unwrap().contains("not found"));
    }

    #[test]
    fn test_response_json() {
        #[derive(Serialize)]
        struct Data {
            name: String,
        }
        let resp = Response::json(&Data { name: "test".to_string() }).unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.headers.get("content-type").unwrap().contains("application/json"));
    }
}
