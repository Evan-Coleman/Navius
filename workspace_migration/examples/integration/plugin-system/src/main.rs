//! Plugin System Integration Example
//!
//! This example demonstrates the integration of plugin system components
//! in the Navius framework, showcasing plugin discovery, loading, and
//! lifecycle management patterns.

use navius_core::error::Result;
use navius_plugin::PluginRegistry;
use std::{path::PathBuf, sync::Arc};
use tracing::{error, info};

use plugin_system::{api::ApiServer, create_plugin_system_app, load_dynamic_plugins};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Plugin System Integration Example");

    // Create the application with plugins
    let app_builder = create_plugin_system_app().await?;
    let mut app = app_builder.build();

    info!("Application built successfully");

    // Get the plugin registry
    let plugin_registry = app.plugin_registry();

    // Try to load dynamic plugins
    let plugins_dir = PathBuf::from("plugins");
    info!("Looking for dynamic plugins in: {:?}", plugins_dir);

    match load_dynamic_plugins(&plugin_registry, plugins_dir).await {
        Ok(loaded_plugins) => {
            if loaded_plugins.is_empty() {
                info!("No dynamic plugins were loaded");
            } else {
                info!("Loaded dynamic plugins: {:?}", loaded_plugins);
            }
        }
        Err(e) => {
            error!("Error loading dynamic plugins: {}", e);
            // Continue even if dynamic plugin loading fails
        }
    }

    // Initialize all plugins (including any newly loaded dynamic plugins)
    plugin_registry.initialize_all().await?;

    // Retrieve components from the registry
    let logger = app
        .get_component_by_name::<Arc<dyn plugin_system::capabilities::LoggingCapability>>(
            "logger",
        )?;
    let config = app
        .get_component_by_name::<Arc<dyn plugin_system::capabilities::ConfigCapability>>(
            "config",
        )?;
    let storage = app
        .get_component_by_name::<Arc<dyn plugin_system::capabilities::StorageCapability>>(
            "storage",
        )?;

    // Create the API server
    let api_server = ApiServer::new(logger.clone(), config.clone(), storage.clone());

    // Configure and start the API server
    api_server.configure_routes()?;

    info!("API Server configured successfully");
    api_server.start().await?;

    info!("Plugin System Integration Example running successfully");

    // Wait for CTRL+C signal
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let mut tx = Some(tx);

    ctrlc::set_handler(move || {
        if let Some(tx) = tx.take() {
            let _ = tx.send(());
        }
    })
    .expect("Error setting Ctrl-C handler");

    rx.await.expect("Error waiting for Ctrl-C");

    info!("Shutdown signal received");

    // Stop the API server
    api_server.stop().await?;

    // Shutdown the application
    plugin_registry.shutdown_all().await?;

    info!("Plugin System Integration Example shut down successfully");
    Ok(())
}
