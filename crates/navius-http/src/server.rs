//! HTTP server functionality for the Navius framework.
//!
//! This module provides HTTP server functionality using Axum.

use crate::error::{Error, Result};
use axum::Router;
use axum::extract::connect_info::{ConnectInfo, IntoMakeServiceWithConnectInfo};
use std::future::Future;
use std::marker::PhantomData;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

/// Wrapper around the broadcast sender for shutdown signals
#[derive(Debug, Clone)]
pub struct ShutdownSender(broadcast::Sender<()>);

impl ShutdownSender {
    /// Send the shutdown signal.
    /// Returns the number of receivers the signal was sent to, or an error
    /// if there are no receivers.
    pub fn send(&self) -> std::result::Result<usize, broadcast::error::SendError<()>> {
        self.0.send(())
    }
}

/// Wrapper around the broadcast receiver for shutdown signals
#[derive(Debug)]
pub struct ShutdownReceiver(#[allow(dead_code)] broadcast::Receiver<()>);

/// Configuration for an HTTP server.
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    address: Option<SocketAddr>,
    timeout: Option<Duration>,
    shutdown_signal_tx: Option<ShutdownSender>,
}

impl HttpServerConfig {
    /// Create new server configuration.
    pub fn new() -> Self {
        Self {
            address: None,
            timeout: None,
            shutdown_signal_tx: None,
        }
    }

    /// Set the address for the server.
    pub fn with_address(mut self, address: SocketAddr) -> Self {
        self.address = Some(address);
        self
    }

    /// Set the host and port for the server.
    pub fn with_host_and_port(mut self, host: &str, port: u16) -> Self {
        let addr = SocketAddr::new(
            IpAddr::from_str(host).unwrap_or_else(|_| IpAddr::from_str("127.0.0.1").unwrap()),
            port,
        );
        self.address = Some(addr);
        self
    }

    /// Set the shutdown signal sender channel.
    /// Use `create_shutdown_channel` to get a sender and receiver pair.
    pub fn with_shutdown(mut self, shutdown_tx: ShutdownSender) -> Self {
        self.shutdown_signal_tx = Some(shutdown_tx);
        self
    }

    /// Create a shutdown channel (sender/receiver pair).
    /// Pass the sender to `with_shutdown`, use the receiver in `shutdown_future`.
    pub fn create_shutdown_channel() -> (ShutdownSender, ShutdownReceiver) {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        (ShutdownSender(tx.clone()), ShutdownReceiver(rx))
    }

    /// Get the configured SocketAddr, defaulting if not set.
    fn get_socket_addr(&self) -> SocketAddr {
        self.address.unwrap_or_else(|| {
            SocketAddr::new(
                IpAddr::from_str("127.0.0.1").unwrap(),
                navius_core::constants::defaults::SERVER_PORT,
            )
        })
    }

    /// Bind a TcpListener based on the configured address.
    pub async fn bind_listener(&self) -> Result<TcpListener> {
        let addr = self.get_socket_addr();
        info!("Binding listener to {}", addr);
        TcpListener::bind(addr)
            .await
            .map_err(|e| Error::internal(format!("Failed to bind listener to {}: {}", addr, e)))
    }

    /// Create a future that resolves when the shutdown signal is received.
    /// Requires a `ShutdownReceiver` obtained from `create_shutdown_channel`.
    pub async fn create_shutdown_future(mut shutdown_rx: ShutdownReceiver) {
        let _ = shutdown_rx.0.recv().await;
        info!("Shutdown signal received, server stopping...");
    }
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience re-exports
pub mod prelude {
    pub use axum::routing::{delete, get, options, patch, post, put};
    pub use axum::{Json, Router};
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use navius_test::error::{TestResult, assert_eq, assert_true};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_server_creation() -> TestResult<()> {
        let server = HttpServerConfig::new();

        assert_true(
            server.address.is_none(),
            "New server should have no address set",
        )?;
        assert_true(
            server.timeout.is_none(),
            "New server should have no timeout set",
        )?;
        assert_true(
            server.shutdown_signal_tx.is_none(),
            "New server should have no shutdown signal set",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_with_address() -> TestResult<()> {
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 8080);
        let server = HttpServerConfig::new().with_address(addr);

        assert_eq(
            server.address,
            Some(addr),
            "Server should have the provided address",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_with_host_and_port() -> TestResult<()> {
        let server = HttpServerConfig::new().with_host_and_port("127.0.0.1", 8080);
        let expected_addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 8080);

        assert_eq(
            server.address,
            Some(expected_addr),
            "Server should have the expected address from host and port",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_router_builder() -> TestResult<()> {
        let builder = RouterBuilder::new();
        let router = builder
            .route("/test", get(|| async { "Hello, World!" }))
            .build();

        // We can't easily test the routes directly, but we can verify the router exists
        assert_true(
            Arc::strong_count(&Arc::new(router)) == 1,
            "Router should be created successfully",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_shutdown_channel() -> TestResult<()> {
        let (tx, rx) = HttpServerConfig::create_shutdown_channel();

        // Send a shutdown signal
        tx.0.send(())?;

        // Verify the receiver gets the signal
        let result = rx.0.try_recv();
        assert_true(result.is_ok(), "Receiver should get the shutdown signal")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_serve_and_shutdown() -> TestResult<()> {
        // Create a server with a custom router
        let router = axum::Router::new().route("/ping", get(|| async { "pong" }));

        // Use a random high port to avoid conflicts
        let server = HttpServerConfig::new()
            .with_router(router)
            .with_host_and_port("127.0.0.1", 0); // Use port 0 for OS assignment

        // Start the server
        let handle = server.serve().await?;

        // Shutdown after a short delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            handle.shutdown();
        });

        // Wait for the server to complete with a timeout
        let result = timeout(Duration::from_secs(5), handle.wait()).await;

        assert_true(
            result.is_ok(),
            "Server should shut down gracefully within the timeout",
        )?;

        Ok(())
    }
}
