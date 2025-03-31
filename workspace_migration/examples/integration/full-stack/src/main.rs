use std::sync::Arc;

use navius_core::tracing::setup_tracing;
use navius_http::server::HttpServerBuilder;
use tokio::signal;

mod api;
mod application;
mod config;
mod domain;
mod infrastructure;
mod plugins;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = config::load_config().await?;

    // Set up tracing
    setup_tracing(&config.log_level, "full-stack-example");

    // Initialize repositories and services (to be implemented)

    // Build and start the HTTP server
    let server = HttpServerBuilder::new()
        .with_config(config.http.clone())
        .build();

    // Configure routes
    let server = api::routes::configure_routes(server);

    // Start the server
    let server_handle = server.start().await?;

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    println!("Shutdown signal received, gracefully shutting down...");

    // Graceful shutdown
    server_handle.shutdown().await;
    println!("Server shutdown completed");

    Ok(())
}
