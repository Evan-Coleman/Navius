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

// Sub-modules
pub mod route_discovery;

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
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a shutdown channel (sender/receiver pair).
/// Use the sender to trigger a shutdown, and the receiver in shutdown_future.
pub fn create_shutdown_channel() -> (ShutdownSender, ShutdownReceiver) {
    let (tx, rx) = tokio::sync::broadcast::channel(1);
    (ShutdownSender(tx.clone()), ShutdownReceiver(rx))
}

/// Bind a TcpListener to the specified address
pub async fn bind_listener(addr: &SocketAddr) -> Result<TcpListener> {
    info!("Binding listener to {}", addr);
    TcpListener::bind(addr)
        .await
        .map_err(|e| Error::internal(format!("Failed to bind listener to {}: {}", addr, e)))
}

/// Create a future that resolves when the shutdown signal is received.
/// Requires a ShutdownReceiver obtained from create_shutdown_channel.
pub async fn shutdown_future(mut shutdown_rx: ShutdownReceiver) {
    let _ = shutdown_rx.0.recv().await;
    info!("Shutdown signal received, server stopping...");
}

/// Convenience re-exports
pub mod prelude {
    pub use axum::routing::{delete, get, options, patch, post, put};
    pub use axum::{Json, Router};
}
