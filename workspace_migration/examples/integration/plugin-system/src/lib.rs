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
use navius_plugin::{Plugin, PluginContext, PluginRegistry};

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
