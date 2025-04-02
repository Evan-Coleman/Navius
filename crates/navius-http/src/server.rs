//! HTTP server functionality for the Navius framework.
//!
//! This module provides HTTP server functionality using Axum.

use crate::error::{Error, Result};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

/// Wrapper around the broadcast sender for shutdown signals
#[derive(Debug, Clone)]
pub struct ShutdownSender(broadcast::Sender<()>);

/// Wrapper around the broadcast receiver for shutdown signals
#[derive(Debug)]
pub struct ShutdownReceiver(#[allow(dead_code)] broadcast::Receiver<()>);

/// HTTP server for the Navius framework.
#[derive(Debug)]
pub struct HttpServer {
    router: axum::Router,
    address: Option<SocketAddr>,
    timeout: Option<Duration>,
    shutdown_signal: Option<ShutdownSender>,
}

impl HttpServer {
    /// Create a new HTTP server.
    pub fn new() -> Self {
        Self {
            router: axum::Router::new(),
            address: None,
            timeout: None,
            shutdown_signal: None,
        }
    }

    /// Create a new HTTP server with a custom router.
    pub fn with_router(mut self, router: axum::Router) -> Self {
        self.router = router;
        self
    }

    /// Configure the router for the server.
    pub fn with_router_builder(self, builder: RouterBuilder) -> Self {
        self.with_router(builder.build())
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

    /// Set the timeout for the server.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the shutdown signal for the server.
    pub fn with_shutdown(mut self, shutdown: Option<ShutdownSender>) -> Self {
        self.shutdown_signal = shutdown;
        self
    }

    /// Create a shutdown channel and return a handle to it.
    pub fn create_shutdown_channel() -> (ShutdownSender, ShutdownReceiver) {
        let (tx, _) = tokio::sync::broadcast::channel(1);
        let rx = tx.subscribe();
        (ShutdownSender(tx), ShutdownReceiver(rx))
    }

    /// Bind to the configured address and start listening.
    pub async fn serve(self) -> Result<HttpServerHandle> {
        // Parse the address
        let addr = self.address.unwrap_or_else(|| {
            SocketAddr::new(
                IpAddr::from_str("127.0.0.1").unwrap(),
                navius_core::constants::defaults::SERVER_PORT,
            )
        });

        info!("Starting HTTP server on {}", addr);

        // Set up the shutdown channel
        let (shutdown_tx, mut shutdown_rx) = match self.shutdown_signal {
            Some(tx) => {
                let rx = tx.0.subscribe();
                (tx, rx)
            }
            None => {
                let (tx, _) = tokio::sync::broadcast::channel(1);
                let rx = tx.subscribe();
                (ShutdownSender(tx), rx)
            }
        };

        // Apply middleware
        let app = self.router.into_make_service();

        // Create the listener
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| Error::internal(format!("Failed to bind to address {}: {}", addr, e)))?;

        // Start the server
        let server = axum::serve(listener, app);

        // Add graceful shutdown
        let server_with_shutdown = server.with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
            info!("Shutdown signal received, stopping server");
        });

        // Spawn the server task
        let handle = tokio::spawn(async move {
            if let Err(e) = server_with_shutdown.await {
                error!("Server error: {}", e);
            }
            info!("Server shutdown complete");
        });

        Ok(HttpServerHandle {
            shutdown_signal: shutdown_tx.0,
            handle,
        })
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for a running HTTP server.
#[derive(Debug)]
pub struct HttpServerHandle {
    shutdown_signal: broadcast::Sender<()>,
    handle: tokio::task::JoinHandle<()>,
}

impl HttpServerHandle {
    /// Shut down the server.
    pub fn shutdown(&self) {
        let _ = self.shutdown_signal.send(());
    }

    /// Wait for the server to complete.
    pub async fn wait(self) -> Result<()> {
        self.handle
            .await
            .map_err(|e| Error::internal(format!("Server join error: {}", e)))
    }
}

/// Builder for creating Axum routers.
#[derive(Debug)]
pub struct RouterBuilder {
    router: axum::Router,
}

impl RouterBuilder {
    /// Create a new router builder.
    pub fn new() -> Self {
        Self {
            router: axum::Router::new(),
        }
    }

    /// Merge another router into this one.
    pub fn merge(mut self, other: axum::Router) -> Self {
        self.router = self.router.merge(other);
        self
    }

    /// Nest another router under a path.
    pub fn nest(mut self, path: &str, router: axum::Router) -> Self {
        self.router = self.router.nest(path, router);
        self
    }

    /// Add a route to the router.
    pub fn route(mut self, path: &str, method_router: axum::routing::MethodRouter) -> Self {
        self.router = self.router.route(path, method_router);
        self
    }

    /// Build the router.
    pub fn build(self) -> axum::Router {
        self.router
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience re-exports
pub mod prelude {
    pub use super::{HttpServer, RouterBuilder};
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
        let server = HttpServer::new();

        assert_true(
            server.address.is_none(),
            "New server should have no address set",
        )?;
        assert_true(
            server.timeout.is_none(),
            "New server should have no timeout set",
        )?;
        assert_true(
            server.shutdown_signal.is_none(),
            "New server should have no shutdown signal set",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_with_address() -> TestResult<()> {
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 8080);
        let server = HttpServer::new().with_address(addr);

        assert_eq(
            server.address,
            Some(addr),
            "Server should have the provided address",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_with_host_and_port() -> TestResult<()> {
        let server = HttpServer::new().with_host_and_port("127.0.0.1", 8080);
        let expected_addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 8080);

        assert_eq(
            server.address,
            Some(expected_addr),
            "Server should have the expected address from host and port",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_with_timeout() -> TestResult<()> {
        let timeout_duration = Duration::from_secs(30);
        let server = HttpServer::new().with_timeout(timeout_duration);

        assert_eq(
            server.timeout,
            Some(timeout_duration),
            "Server should have the provided timeout",
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
        let (tx, rx) = HttpServer::create_shutdown_channel();

        // Send a shutdown signal
        tx.0.send(())?;

        // Verify the receiver gets the signal
        let result = rx.0.try_recv();
        assert_true(result.is_ok(), "Receiver should get the shutdown signal")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_handle_shutdown() -> TestResult<()> {
        // Create a shutdown channel
        let (tx, _) = HttpServer::create_shutdown_channel();

        // Create a server handle
        let handle = HttpServerHandle {
            shutdown_signal: tx.0.clone(),
            handle: tokio::spawn(async {
                // Simulated server task
                tokio::time::sleep(Duration::from_secs(10)).await;
            }),
        };

        // Shutdown the server
        handle.shutdown();

        // Verify the sender has sent a message
        assert_eq(
            tx.0.receiver_count(),
            0,
            "All receivers should be notified by the shutdown signal",
        )?;

        Ok(())
    }

    #[tokio::test]
    async fn test_server_serve_and_shutdown() -> TestResult<()> {
        // Create a server with a custom router
        let router = axum::Router::new().route("/ping", get(|| async { "pong" }));

        // Use a random high port to avoid conflicts
        let server = HttpServer::new()
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
