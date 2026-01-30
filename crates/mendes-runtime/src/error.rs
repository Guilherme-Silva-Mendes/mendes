//! Mendes runtime errors

use std::fmt;

/// Standard Mendes result type
pub type Result<T> = std::result::Result<T, MendesError>;

/// Runtime errors
#[derive(Debug)]
pub enum MendesError {
    /// HTTP error
    Http(HttpError),
    /// Database error
    Database(String),
    /// I/O error
    Io(std::io::Error),
    /// Serialization error
    Serialization(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for MendesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MendesError::Http(e) => write!(f, "HTTP error: {}", e),
            MendesError::Database(msg) => write!(f, "Database error: {}", msg),
            MendesError::Io(e) => write!(f, "IO error: {}", e),
            MendesError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            MendesError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for MendesError {}

impl From<std::io::Error> for MendesError {
    fn from(e: std::io::Error) -> Self {
        MendesError::Io(e)
    }
}

impl From<hyper::Error> for MendesError {
    fn from(e: hyper::Error) -> Self {
        MendesError::Http(HttpError::new(500, e.to_string()))
    }
}

impl From<serde_json::Error> for MendesError {
    fn from(e: serde_json::Error) -> Self {
        MendesError::Serialization(e.to_string())
    }
}

/// HTTP error with status code
#[derive(Debug, Clone)]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

impl HttpError {
    pub fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(401, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(403, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.status, self.message)
    }
}

impl std::error::Error for HttpError {}

impl From<HttpError> for MendesError {
    fn from(e: HttpError) -> Self {
        MendesError::Http(e)
    }
}
