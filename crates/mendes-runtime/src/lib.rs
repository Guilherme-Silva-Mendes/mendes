//! mendes-runtime - Mendes language runtime
//!
//! Provides:
//! - **HTTP Server**: High-performance asynchronous HTTP server
//! - **Router**: Request routing with path parameter support
//! - **Database Pool**: Connection pool for PostgreSQL, MySQL and SQLite
//! - **Async Runtime**: Asynchronous executor based on Tokio
//!
//! # Example
//!
//! ```rust,ignore
//! use mendes_runtime::{Server, Router, Request, Response};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = Router::new();
//!     router.get("/health", |_req| async {
//!         Response::ok("healthy")
//!     });
//!
//!     Server::new("0.0.0.0:8080")
//!         .router(router)
//!         .run()
//!         .await;
//! }
//! ```

pub mod http;
pub mod router;
pub mod database;
pub mod types;
pub mod error;
pub mod middleware;
pub mod websocket;

pub use http::{Server, Request, Response, StatusCode};
pub use router::Router;
pub use types::{MendesString, MendesArray, MendesResult, MendesOption};
pub use websocket::WsConnection;
pub use error::{MendesError, Result};

#[cfg(feature = "postgres")]
pub use database::PostgresPool;

#[cfg(feature = "mysql")]
pub use database::MysqlPool;

#[cfg(feature = "sqlite")]
pub use database::SqlitePool;

/// Re-export tokio for use in generated code
pub use tokio;

/// Initializes the Mendes runtime
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
}

/// Macro to create HTTP handler
#[macro_export]
macro_rules! handler {
    ($name:ident, $body:expr) => {
        pub async fn $name(req: $crate::Request) -> $crate::Response {
            $body
        }
    };
}

/// Macro to start server
#[macro_export]
macro_rules! server {
    ($host:expr, $port:expr, $router:expr) => {
        $crate::Server::new(&format!("{}:{}", $host, $port))
            .router($router)
            .run()
            .await
    };
}
