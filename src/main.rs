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
    setup_tracing(&config.log_level, &config.app_name);

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
    
    tracing::info!("Server started at http://{}:{}", config.http.host, config.http.port);

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received, gracefully shutting down...");

    // Graceful shutdown
    server_handle.shutdown().await;
    tracing::info!("Server shutdown completed");

    Ok(())
}

/// Initialize all services based on configuration
async fn initialize_services(
    config: &config::AppConfig,
) -> Result<Arc<infrastructure::ServiceRegistry>, Box<dyn std::error::Error>> {
    // Create a new service registry
    let mut registry = infrastructure::ServiceRegistry::new();
    
    // Initialize application services
    application::init();
    
    // Initialize database connection if configured
    #[cfg(feature = "database")]
    if let Some(db_config) = &config.database {
        if db_config.enabled {
            let db_pool = infrastructure::init_database(db_config).await?;
            registry = registry.with_db_pool(db_pool);
            tracing::info!("Database connection initialized");
        }
    }
    
    // Initialize cache connection if configured
    #[cfg(feature = "cache")]
    if let Some(cache_config) = &config.cache {
        if cache_config.enabled {
            let cache_client = infrastructure::init_cache(cache_config).await?;
            registry = registry.with_cache_client(cache_client);
            tracing::info!("Cache connection initialized");
        }
    }
    
    // Initialize authentication service if configured
    #[cfg(feature = "entra-auth")]
    application::init_auth_service(config, &mut registry).await?;
    
    // Initialize plugin system if enabled
    #[cfg(feature = "plugin")]
    application::init_plugin_system(&mut registry).await?;
    
    // Initialize event system if enabled
    #[cfg(feature = "event")]
    application::init_event_system(&mut registry).await?;
    
    // Initialize metrics if enabled
    #[cfg(feature = "metrics")]
    application::init_metrics(&mut registry).await?;
    
    // Return the initialized service registry
    Ok(Arc::new(registry))
}
