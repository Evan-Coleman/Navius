use async_trait::async_trait;
use semver::Version;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::error::{PluginError, PluginHealth, PluginResult};
use crate::plugin::{Plugin, PluginConfig, PluginLifecycleStage};

/// Manages a collection of plugins and their lifecycle
pub struct PluginRegistry {
    /// Map of plugin ID to plugin instance
    plugins: RwLock<HashMap<String, Arc<RwLock<Box<dyn Plugin>>>>>,

    /// Dependencies between plugins (plugin_id -> vec of dependency_ids)
    dependencies: RwLock<HashMap<String, Vec<String>>>,

    /// Starting order for plugins
    startup_order: RwLock<Vec<String>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            dependencies: RwLock::new(HashMap::new()),
            startup_order: RwLock::new(Vec::new()),
        }
    }

    /// Register a plugin with the registry
    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> PluginResult<String> {
        let plugin_id = plugin.id().to_string();

        // Check if plugin is already registered
        if self.plugins.read().unwrap().contains_key(&plugin_id) {
            return Err(PluginError::PluginAlreadyRegistered(plugin_id));
        }

        // Register the plugin
        let plugin = Arc::new(RwLock::new(plugin));
        self.plugins
            .write()
            .unwrap()
            .insert(plugin_id.clone(), plugin);

        Ok(plugin_id)
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        self.plugins.read().unwrap().get(plugin_id).cloned()
    }

    /// Get all registered plugins
    pub fn get_all_plugins(&self) -> Vec<(String, Arc<RwLock<Box<dyn Plugin>>>)> {
        self.plugins
            .read()
            .unwrap()
            .iter()
            .map(|(id, plugin)| (id.clone(), plugin.clone()))
            .collect()
    }

    /// Unregister a plugin by ID
    pub async fn unregister_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        // Check if plugin exists
        if !self.plugins.read().unwrap().contains_key(plugin_id) {
            return Err(PluginError::PluginNotFound(plugin_id.to_string()));
        }

        // Check if any plugins depend on this one
        for (dep_id, deps) in self.dependencies.read().unwrap().iter() {
            if deps.contains(&plugin_id.to_string()) {
                return Err(PluginError::DependencyError(format!(
                    "Cannot unregister plugin '{}' because plugin '{}' depends on it",
                    plugin_id, dep_id
                )));
            }
        }

        // Remove from dependencies and startup order
        self.dependencies.write().unwrap().remove(plugin_id);
        if let Ok(mut order) = self.startup_order.write() {
            if let Some(pos) = order.iter().position(|id| id == plugin_id) {
                order.remove(pos);
            }
        }

        // Stop the plugin if it's running
        let plugin_opt = self.plugins.read().unwrap().get(plugin_id).cloned();
        if let Some(plugin) = plugin_opt {
            let mut plugin_guard = plugin.write().unwrap();
            if plugin_guard.lifecycle_stage() == PluginLifecycleStage::Started {
                plugin_guard.stop().await?;
            }
        }

        // Remove the plugin from the registry
        self.plugins.write().unwrap().remove(plugin_id);

        Ok(())
    }

    /// Initialize a plugin by ID with the given configuration
    pub async fn initialize_plugin(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> PluginResult<()> {
        let plugin_opt = self.get_plugin(plugin_id);

        if let Some(plugin) = plugin_opt {
            let mut plugin_guard = plugin.write().unwrap();

            // Check if the plugin is in the correct state
            if plugin_guard.lifecycle_stage() != PluginLifecycleStage::Created {
                return Err(PluginError::InvalidState {
                    plugin_id: plugin_id.to_string(),
                    current_state: plugin_guard.lifecycle_stage().to_string(),
                    required_state: PluginLifecycleStage::Created.to_string(),
                });
            }

            // Initialize the plugin
            plugin_guard.initialize(config).await
        } else {
            Err(PluginError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Start a plugin by ID
    pub async fn start_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let plugin_opt = self.get_plugin(plugin_id);

        if let Some(plugin) = plugin_opt {
            let mut plugin_guard = plugin.write().unwrap();

            // Check if the plugin is in the correct state
            if plugin_guard.lifecycle_stage() != PluginLifecycleStage::Initialized {
                return Err(PluginError::InvalidState {
                    plugin_id: plugin_id.to_string(),
                    current_state: plugin_guard.lifecycle_stage().to_string(),
                    required_state: PluginLifecycleStage::Initialized.to_string(),
                });
            }

            // Start the plugin
            plugin_guard.start().await
        } else {
            Err(PluginError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Stop a plugin by ID
    pub async fn stop_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let plugin_opt = self.get_plugin(plugin_id);

        if let Some(plugin) = plugin_opt {
            let mut plugin_guard = plugin.write().unwrap();

            // Check if the plugin is in the correct state
            if plugin_guard.lifecycle_stage() != PluginLifecycleStage::Started {
                return Err(PluginError::InvalidState {
                    plugin_id: plugin_id.to_string(),
                    current_state: plugin_guard.lifecycle_stage().to_string(),
                    required_state: PluginLifecycleStage::Started.to_string(),
                });
            }

            // Stop the plugin
            plugin_guard.stop().await
        } else {
            Err(PluginError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Check the health of a plugin by ID
    pub async fn health_check_plugin(&self, plugin_id: &str) -> PluginResult<PluginHealth> {
        let plugin_opt = self.get_plugin(plugin_id);

        if let Some(plugin) = plugin_opt {
            let plugin_guard = plugin.read().unwrap();
            Ok(plugin_guard.health_check().await)
        } else {
            Err(PluginError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Check the health of all plugins
    pub async fn health_check_all(&self) -> HashMap<String, PluginHealth> {
        let mut results = HashMap::new();

        for (id, plugin) in self.get_all_plugins() {
            let plugin_guard = plugin.read().unwrap();
            results.insert(id, plugin_guard.health_check().await);
        }

        results
    }

    /// Resolve plugin dependencies
    pub fn resolve_dependencies(&self) -> PluginResult<()> {
        let plugins = self.plugins.read().unwrap().clone();
        let mut dependencies = HashMap::new();
        let mut satisfied = true;

        // Collect dependencies from all plugins
        for (id, plugin) in &plugins {
            let plugin_guard = plugin.read().unwrap();
            let deps: Vec<String> = plugin_guard
                .dependencies()
                .into_iter()
                .filter_map(|dep| {
                    // Check if the dependency exists
                    match plugins.get(&dep.id) {
                        Some(dep_plugin) => {
                            let dep_plugin_guard = dep_plugin.read().unwrap();
                            let dep_version =
                                match Version::parse(&dep_plugin_guard.metadata().version) {
                                    Ok(v) => v,
                                    Err(_) => {
                                        satisfied = false;
                                        return Some(dep.id);
                                    }
                                };

                            let req_version = match Version::parse(&dep.version) {
                                Ok(v) => v,
                                Err(_) => {
                                    satisfied = false;
                                    return Some(dep.id);
                                }
                            };

                            // Check version compatibility
                            if dep_version.major != req_version.major
                                || dep_version.minor < req_version.minor
                            {
                                satisfied = false;
                            }

                            Some(dep.id)
                        }
                        None if dep.optional => None,
                        None => {
                            satisfied = false;
                            Some(dep.id)
                        }
                    }
                })
                .collect();

            dependencies.insert(id.clone(), deps);
        }

        if !satisfied {
            return Err(PluginError::DependencyError(
                "Not all dependencies are satisfied".to_string(),
            ));
        }

        // Check for circular dependencies
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for id in dependencies.keys() {
            if !visited.contains(id)
                && has_circular_dependency(id, &dependencies, &mut visited, &mut stack)
            {
                return Err(PluginError::CircularDependency(
                    "Circular dependency detected".to_string(),
                ));
            }
        }

        // Store dependencies
        *self.dependencies.write().unwrap() = dependencies.clone();

        // Calculate startup order
        let mut order = Vec::new();
        visited.clear();

        for id in dependencies.keys() {
            if !visited.contains(id) {
                topological_sort(id, &dependencies, &mut visited, &mut order);
            }
        }

        // Reverse the order for proper startup sequence
        order.reverse();
        *self.startup_order.write().unwrap() = order;

        Ok(())
    }

    /// Start all plugins in dependency order
    pub async fn start_all_plugins(&self) -> PluginResult<()> {
        // Ensure dependencies are resolved
        self.resolve_dependencies()?;

        // Get startup order
        let order = self.startup_order.read().unwrap().clone();

        // Start plugins in order
        for plugin_id in order {
            self.start_plugin(&plugin_id).await?;
        }

        Ok(())
    }

    /// Stop all plugins in reverse dependency order
    pub async fn stop_all_plugins(&self) -> PluginResult<()> {
        // Get startup order (reverse it for shutdown)
        let mut order = self.startup_order.read().unwrap().clone();
        order.reverse();

        // Stop plugins in order
        for plugin_id in order {
            // Ignore errors when stopping plugins during shutdown
            let _ = self.stop_plugin(&plugin_id).await;
        }

        Ok(())
    }

    /// Get a plugin capability by ID and capability ID
    pub fn get_plugin_capability(
        &self,
        plugin_id: &str,
        capability_id: &str,
    ) -> PluginResult<Arc<dyn std::any::Any + Send + Sync>> {
        let plugin_opt = self.get_plugin(plugin_id);

        if let Some(plugin) = plugin_opt {
            let plugin_guard = plugin.read().unwrap();

            if let Some(capability) = plugin_guard.get_capability(capability_id) {
                Ok(capability)
            } else {
                Err(PluginError::CapabilityNotFound {
                    plugin_id: plugin_id.to_string(),
                    capability_id: capability_id.to_string(),
                })
            }
        } else {
            Err(PluginError::PluginNotFound(plugin_id.to_string()))
        }
    }

    /// Find plugins by tag
    pub fn find_plugins_by_tag(&self, tag: &str) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();

        plugins
            .iter()
            .filter_map(|(id, plugin)| {
                let plugin_guard = plugin.read().unwrap();
                if plugin_guard.metadata().tags.contains(&tag.to_string()) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager trait for plugin registries
#[async_trait]
pub trait PluginRegistryManager: Send + Sync {
    /// Get a plugin registry
    fn get_registry(&self) -> Arc<PluginRegistry>;

    /// Register a plugin
    async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> PluginResult<String>;

    /// Get a plugin by ID
    fn get_plugin(&self, plugin_id: &str) -> Option<Arc<RwLock<Box<dyn Plugin>>>>;

    /// Start a plugin by ID
    async fn start_plugin(&self, plugin_id: &str) -> PluginResult<()>;

    /// Stop a plugin by ID
    async fn stop_plugin(&self, plugin_id: &str) -> PluginResult<()>;

    /// Check the health of a plugin by ID
    async fn health_check_plugin(&self, plugin_id: &str) -> PluginResult<PluginHealth>;

    /// Get a plugin capability by ID and capability ID
    fn get_plugin_capability(
        &self,
        plugin_id: &str,
        capability_id: &str,
    ) -> PluginResult<Arc<dyn std::any::Any + Send + Sync>>;
}

/// Utility function to check for circular dependencies
fn has_circular_dependency(
    node: &str,
    dependencies: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> bool {
    visited.insert(node.to_string());
    stack.push(node.to_string());

    if let Some(deps) = dependencies.get(node) {
        for dep in deps {
            if !visited.contains(dep) {
                if has_circular_dependency(dep, dependencies, visited, stack) {
                    return true;
                }
            } else if stack.contains(dep) {
                return true;
            }
        }
    }

    stack.pop();
    false
}

/// Utility function for topological sorting
fn topological_sort(
    node: &str,
    dependencies: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    order: &mut Vec<String>,
) {
    visited.insert(node.to_string());

    if let Some(deps) = dependencies.get(node) {
        for dep in deps {
            if !visited.contains(dep) {
                topological_sort(dep, dependencies, visited, order);
            }
        }
    }

    order.push(node.to_string());
}
