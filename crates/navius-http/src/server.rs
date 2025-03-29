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
pub struct ShutdownReceiver(broadcast::Receiver<()>);

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
