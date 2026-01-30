//! WebSocket support for Mendes runtime
//!
//! Provides WebSocket connections for real-time communication.

use std::sync::Arc;
use tokio::sync::mpsc;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;

/// WebSocket connection handle
#[derive(Clone)]
pub struct WsConnection {
    /// Sender for outgoing messages
    sender: mpsc::UnboundedSender<String>,
    /// Connection ID
    pub id: String,
}

impl WsConnection {
    /// Creates a new WebSocket connection
    pub fn new(sender: mpsc::UnboundedSender<String>) -> Self {
        Self {
            sender,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Sends a message to the client
    pub async fn send(&self, message: &str) {
        let _ = self.sender.send(message.to_string());
    }

    /// Sends a JSON message to the client
    pub async fn send_json<T: serde::Serialize>(&self, data: &T) {
        if let Ok(json) = serde_json::to_string(data) {
            let _ = self.sender.send(json);
        }
    }

    /// Broadcasts a message to all connections in a room
    pub async fn broadcast(&self, _room: &str, _message: &str) {
        // TODO: Implement room-based broadcasting
    }

    /// Joins a room
    pub async fn join(&self, _room: &str) {
        // TODO: Implement room joining
    }

    /// Leaves a room
    pub async fn leave(&self, _room: &str) {
        // TODO: Implement room leaving
    }
}

/// WebSocket handler trait
pub trait WsHandler: Send + Sync + 'static {
    /// Called when a new connection is established
    fn on_connect(&self, conn: WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

    /// Called when a message is received
    fn on_message(&self, conn: WsConnection, message: String) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

    /// Called when the connection is closed
    fn on_disconnect(&self, conn: WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;
}

/// Simple WebSocket handler using closures
pub struct SimplerWsHandler<F, M, D>
where
    F: Fn(WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    M: Fn(WsConnection, String) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    D: Fn(WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    on_connect: F,
    on_message: M,
    on_disconnect: D,
}

impl<F, M, D> SimplerWsHandler<F, M, D>
where
    F: Fn(WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    M: Fn(WsConnection, String) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    D: Fn(WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    pub fn new(on_connect: F, on_message: M, on_disconnect: D) -> Self {
        Self {
            on_connect,
            on_message,
            on_disconnect,
        }
    }
}

impl<F, M, D> WsHandler for SimplerWsHandler<F, M, D>
where
    F: Fn(WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    M: Fn(WsConnection, String) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    D: Fn(WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    fn on_connect(&self, conn: WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        (self.on_connect)(conn)
    }

    fn on_message(&self, conn: WsConnection, message: String) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        (self.on_message)(conn, message)
    }

    fn on_disconnect(&self, conn: WsConnection) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
        (self.on_disconnect)(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ws_connection_send() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let conn = WsConnection::new(tx);

        conn.send("Hello").await;

        let msg = rx.recv().await;
        assert_eq!(msg, Some("Hello".to_string()));
    }
}
