// Copyright (c) 2025 Navius Contributors
//
// Licensed under the MIT License or the Apache License, Version 2.0,
// at your option.

use std::sync::Arc;

use navius_core::tracing::setup_tracing;
use navius_http::server::HttpServerBuilder;
use tokio::signal;

mod api;
mod application;
mod config;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = config::load_config().await?;

    // Set up tracing
    setup_tracing(&config.log_level, "navius");

    // Initialize the service registry
    let service_registry = initialize_services(&config).await?;

    // Build and start the HTTP server
    let server = HttpServerBuilder::new()
        .with_config(config.http.clone())
        .build();

    // Configure routes
    let server = api::configure_routes(server, service_registry.clone());

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

async fn initialize_services(
    config: &config::AppConfig,
) -> Result<Arc<infrastructure::ServiceRegistry>, Box<dyn std::error::Error>> {
    // Create a new service registry
    let registry = infrastructure::ServiceRegistry::new();

    // Initialize application services
    application::init();

    // Initialize database connection if configured
    // (This would be implemented based on the application requirements)

    // Initialize cache connection if configured
    // (This would be implemented based on the application requirements)

    // Return the initialized service registry
    Ok(Arc::new(registry))
}
