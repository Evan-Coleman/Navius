//! Plugin system for the Navius framework
//!
//! This module provides a lightweight plugin system for the Navius framework.
//! Plugins are reusable components that can be registered with the application
//! and have well-defined lifecycle hooks.

use std::collections::HashSet;
use std::fmt::Debug;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::future::BoxFuture;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::component::ComponentRegistry;
use crate::error::{PluginError, PluginResult};

/// Plugin trait that all plugins must implement.
///
/// Plugins are the core extension mechanism in the framework, allowing
/// for modular functionality that can be composed together.
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Returns the plugin ID.
    fn id(&self) -> &str;

    /// Returns the plugin description.
    fn description(&self) -> &str;

    /// Returns the IDs of plugins this plugin depends on.
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// Initializes the plugin.
    ///
    /// This method is called during the startup sequence.
    #[instrument(skip(self), fields(plugin_id = self.id()))]
    async fn initialize(&self, _registry: &ComponentRegistry) -> PluginResult<()> {
        trace!("Default initialization - no custom logic");
        Ok(())
    }

    /// Shuts down the plugin.
    ///
    /// This method is called during the shutdown sequence.
    #[instrument(skip(self), fields(plugin_id = self.id()))]
    async fn shutdown(&self, _registry: &ComponentRegistry) -> PluginResult<()> {
        trace!("Default shutdown - no custom logic");
        Ok(())
    }
}

/// Registry for managing plugins.
///
/// The plugin registry is responsible for managing plugin instances,
/// initialization order, and the component registry.
pub struct PluginRegistry {
    /// Component registry shared with all plugins.
    component_registry: ComponentRegistry,
    /// Map of plugin IDs to plugin instances.
    plugins: DashMap<String, Box<dyn Plugin>>,
    /// Map of plugin IDs to initialization status.
    initialized: DashMap<String, bool>,
}

impl PluginRegistry {
    /// Creates a new plugin registry.
    pub fn new() -> Self {
        Self {
            component_registry: ComponentRegistry::new(),
            plugins: DashMap::new(),
            initialized: DashMap::new(),
        }
    }

    /// Creates a new plugin registry with an existing component registry.
    pub fn with_component_registry(component_registry: ComponentRegistry) -> Self {
        Self {
            component_registry,
            plugins: DashMap::new(),
            initialized: DashMap::new(),
        }
    }

    /// Returns a reference to the component registry.
    pub fn component_registry(&self) -> &ComponentRegistry {
        &self.component_registry
    }

    /// Registers a plugin with the registry.
    ///
    /// Returns an error if a plugin with the same ID is already registered.
    #[instrument(skip(self, plugin), fields(plugin_id = plugin.id()))]
    pub fn register_plugin<P: Plugin>(&self, plugin: P) -> PluginResult<()> {
        let plugin_id = plugin.id().to_string();

        if self.plugins.contains_key(&plugin_id) {
            warn!("Duplicate plugin registration attempt");
            return Err(PluginError::DuplicatePlugin {
                plugin_id: plugin_id.clone(),
            });
        }

        debug!("Registering plugin: {}", plugin_id);
        self.plugins.insert(plugin_id.clone(), Box::new(plugin));
        self.initialized.insert(plugin_id, false);

        trace!("Plugin registered successfully");
        Ok(())
    }

    /// Checks if a plugin is registered.
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }

    /// Returns the number of registered plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Gets the initialization status of a plugin.
    pub fn is_initialized(&self, plugin_id: &str) -> bool {
        self.initialized
            .get(plugin_id)
            .map_or(false, |status| *status)
    }

    /// Determines the initialization order for plugins based on their dependencies.
    ///
    /// Returns a result containing the ordered list of plugin IDs or an error if
    /// there are missing dependencies or a circular dependency is detected.
    #[instrument(skip(self))]
    fn determine_initialization_order(&self) -> PluginResult<Vec<String>> {
        debug!("Determining plugin initialization order");

        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();

        // Helper function for topological sort with cycle detection
        // Using a non-async function to avoid recursion issues in async functions
        fn visit(
            plugin_id: &str,
            plugin_registry: &PluginRegistry,
            visited: &mut HashSet<String>,
            temp_visited: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) -> PluginResult<()> {
            // Check for circular dependency
            if temp_visited.contains(plugin_id) {
                return Err(PluginError::CircularDependency {
                    plugin_id: plugin_id.to_string(),
                });
            }

            // Skip if already visited
            if visited.contains(plugin_id) {
                return Ok(());
            }

            // Mark as temporarily visited for cycle detection
            temp_visited.insert(plugin_id.to_string());

            // Get the plugin and its dependencies
            if let Some(plugin) = plugin_registry.plugins.get(plugin_id) {
                // Visit dependencies first
                for dep_id in plugin.dependencies() {
                    // Check if dependency exists
                    if !plugin_registry.has_plugin(&dep_id) {
                        return Err(PluginError::DependencyNotFound {
                            plugin_id: plugin_id.to_string(),
                            dependency_id: dep_id,
                        });
                    }

                    // Visit dependency
                    visit(&dep_id, plugin_registry, visited, temp_visited, result)?;
                }
            } else {
                return Err(PluginError::DependencyNotFound {
                    plugin_id: "unknown".to_string(),
                    dependency_id: plugin_id.to_string(),
                });
            }

            // Remove from temporary visited as we're done with this node
            temp_visited.remove(plugin_id);

            // Mark as visited and add to result
            visited.insert(plugin_id.to_string());
            result.push(plugin_id.to_string());

            Ok(())
        }

        // Process all plugins to ensure we handle disconnected subgraphs
        for plugin_id in self.plugins.iter().map(|entry| entry.key().clone()) {
            visit(
                &plugin_id,
                self,
                &mut visited,
                &mut temp_visited,
                &mut result,
            )?;
        }

        debug!("Plugin initialization order determined: {:?}", result);
        Ok(result)
    }

    /// Initializes all registered plugins in dependency order.
    ///
    /// Returns a result indicating success or failure.
    #[instrument(skip(self))]
    pub async fn initialize_all(&self) -> PluginResult<()> {
        debug!("Initializing all plugins");

        // Get initialization order
        let init_order = self.determine_initialization_order()?;

        // Initialize plugins in order
        for plugin_id in init_order {
            self.initialize_plugin(&plugin_id).await?;
        }

        info!("All plugins initialized successfully");
        Ok(())
    }

    /// Initializes a single plugin and its dependencies.
    ///
    /// Returns a result indicating success or failure.
    #[instrument(skip(self), fields(plugin_id = id))]
    pub async fn initialize_plugin(&self, id: &str) -> PluginResult<()> {
        // Skip if already initialized
        if self.is_initialized(id) {
            trace!("Plugin already initialized, skipping");
            return Ok(());
        }

        // Get the plugin
        let plugin = self
            .plugins
            .get(id)
            .ok_or_else(|| PluginError::DependencyNotFound {
                plugin_id: "requested".to_string(),
                dependency_id: id.to_string(),
            })?;

        // Initialize dependencies first
        for dep_id in plugin.dependencies() {
            if !self.is_initialized(&dep_id) {
                // Use a helper function to avoid recursion in async functions
                self.initialize_plugin_helper(&dep_id).await?;
            }
        }

        // Initialize the plugin
        debug!("Initializing plugin: {}", id);
        plugin.initialize(&self.component_registry).await?;

        // Mark as initialized
        self.initialized.insert(id.to_string(), true);

        debug!("Plugin initialized successfully: {}", id);
        Ok(())
    }

    // Helper function to avoid recursive async fn
    fn initialize_plugin_helper<'a>(
        &'a self,
        plugin_id: &'a str,
    ) -> BoxFuture<'a, PluginResult<()>> {
        Box::pin(async move { self.initialize_plugin(plugin_id).await })
    }

    /// Shuts down all plugins in reverse initialization order.
    ///
    /// Returns a result indicating success or failure.
    #[instrument(skip(self))]
    pub async fn shutdown_all(&self) -> PluginResult<()> {
        debug!("Shutting down all plugins");

        // Get initialization order and reverse it for shutdown
        let init_order = self.determine_initialization_order()?;
        let shutdown_order: Vec<String> = init_order.into_iter().rev().collect();

        // Track any errors that occur during shutdown
        let mut errors = Vec::new();

        // Shutdown plugins in reverse order
        for plugin_id in shutdown_order {
            // Only shut down if it was initialized
            if self.is_initialized(&plugin_id) {
                if let Some(plugin) = self.plugins.get(&plugin_id) {
                    debug!("Shutting down plugin: {}", plugin_id);

                    // Attempt to shut down, but collect errors instead of returning early
                    match plugin.shutdown(&self.component_registry).await {
                        Ok(_) => {
                            // Mark as not initialized
                            self.initialized.insert(plugin_id.clone(), false);
                            debug!("Plugin shutdown successfully: {}", plugin_id);
                        }
                        Err(e) => {
                            error!("Error shutting down plugin {}: {}", plugin_id, e);
                            errors.push((plugin_id.clone(), e));
                        }
                    }
                }
            }
        }

        // If there were any errors, return an error with details
        if !errors.is_empty() {
            return Err(PluginError::ShutdownError {
                errors: errors
                    .into_iter()
                    .map(|(id, err)| format!("{}: {}", id, err))
                    .collect(),
            });
        }

        info!("All plugins shut down successfully");
        Ok(())
    }

    /// Removes a plugin from the registry.
    ///
    /// Returns true if the plugin was removed, false if it wasn't found.
    #[instrument(skip(self), fields(plugin_id = plugin_id))]
    pub fn remove_plugin(&self, plugin_id: &str) -> bool {
        self.initialized.remove(plugin_id);
        self.plugins.remove(plugin_id).is_some()
    }

    /// Clears all plugins from the registry.
    #[instrument(skip(self))]
    pub fn clear(&self) {
        self.plugins.clear();
        self.initialized.clear();
        debug!("Plugin registry cleared");
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("plugin_count", &self.plugins.len())
            .field("initialized_count", &self.initialized.len())
            .finish()
    }
}

/// Builder for creating plugins.
pub struct PluginBuilder {
    /// Plugin ID.
    id: String,
    /// Plugin description.
    description: String,
    /// Plugin dependencies.
    dependencies: Vec<String>,
}

impl PluginBuilder {
    /// Creates a new plugin builder with the given ID.
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            description: String::new(),
            dependencies: Vec::new(),
        }
    }

    /// Sets the plugin description.
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = description.into();
        self
    }

    /// Adds a dependency to the plugin.
    pub fn dependency<S: Into<String>>(mut self, dependency: S) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Adds multiple dependencies to the plugin.
    pub fn dependencies<S: Into<String>, I: IntoIterator<Item = S>>(
        mut self,
        dependencies: I,
    ) -> Self {
        self.dependencies
            .extend(dependencies.into_iter().map(Into::into));
        self
    }

    /// Builds a simple plugin instance.
    pub fn build(self) -> SimplePlugin {
        SimplePlugin {
            id: self.id,
            description: self.description,
            dependencies: self.dependencies,
        }
    }
}

/// A simple plugin implementation.
///
/// This implementation provides the basic plugin functionality with no custom behavior.
#[derive(Clone)]
pub struct SimplePlugin {
    /// Plugin ID.
    id: String,
    /// Plugin description.
    description: String,
    /// Plugin dependencies.
    dependencies: Vec<String>,
}

#[async_trait]
impl Plugin for SimplePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    #[instrument(skip(self), fields(plugin_id = self.id()))]
    async fn initialize(&self, _registry: &ComponentRegistry) -> PluginResult<()> {
        trace!("Simple plugin initialized");
        Ok(())
    }

    #[instrument(skip(self), fields(plugin_id = self.id()))]
    async fn shutdown(&self, _registry: &ComponentRegistry) -> PluginResult<()> {
        trace!("Simple plugin shut down");
        Ok(())
    }
}
