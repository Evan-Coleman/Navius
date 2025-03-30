//! Plugin System Integration Example Library
//!
//! This crate demonstrates the integration of plugin system components
//! in the Navius framework, showcasing plugin discovery, loading, and
//! lifecycle management patterns.

pub mod api;
pub mod capabilities;
pub mod config;
pub mod plugins;
pub mod services;

use async_trait::async_trait;
use navius_core::{
    di::{Application, ApplicationBuilder, ComponentScope, Environment},
    error::Result,
};
use navius_plugin::{Plugin, PluginContext, PluginLoader, PluginRegistry};

use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Create and configure a plugin system application
pub async fn create_plugin_system_app() -> Result<ApplicationBuilder> {
    // Create plugin registry
    let plugin_registry = Arc::new(PluginRegistry::new());

    // Create application builder
    let mut app_builder = ApplicationBuilder::new()
        .with_environment(Environment::Development)
        .with_plugin_registry(plugin_registry.clone());

    // Load and register plugins from the plugins module
    // This would typically happen automatically through plugin discovery
    plugins::register_all_plugins(&plugin_registry)?;

    // Initialize all plugins
    plugin_registry.initialize_all().await?;

    Ok(app_builder)
}

/// Load dynamic plugins from a directory
pub async fn load_dynamic_plugins(
    registry: &Arc<PluginRegistry>,
    plugin_dir: impl AsRef<Path>,
) -> Result<Vec<String>> {
    let mut plugin_loader = PluginLoader::new();
    plugin_loader.add_search_path(plugin_dir);

    info!(
        "Searching for dynamic plugins in: {:?}",
        plugin_loader.search_paths()
    );

    let plugin_paths = plugin_loader.find_plugins()?;
    if plugin_paths.is_empty() {
        info!("No dynamic plugins found");
        return Ok(Vec::new());
    }

    info!("Found {} dynamic plugin(s)", plugin_paths.len());

    let mut loaded_plugin_names = Vec::new();
    for path in plugin_paths {
        info!("Loading dynamic plugin from: {:?}", path);
        match plugin_loader.load_plugin(&path) {
            Ok(plugin) => {
                let plugin_name = plugin.name().to_string();
                info!("Loaded dynamic plugin: {}", plugin_name);
                registry.register(plugin)?;
                loaded_plugin_names.push(plugin_name);
            }
            Err(e) => {
                tracing::error!("Failed to load dynamic plugin from {:?}: {}", path, e);
            }
        }
    }

    Ok(loaded_plugin_names)
}
