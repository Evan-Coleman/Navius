use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::MessageHandler;
use crate::capability::Capability;
use crate::error::{PluginError, PluginHealth, PluginResult};
use crate::plugin::{Plugin, PluginConfig, PluginDependency, PluginLifecycleStage, PluginMetadata};

/// A simple base implementation of the Plugin trait
#[derive(Debug)]
pub struct BasePlugin {
    /// Plugin metadata
    metadata: PluginMetadata,

    /// Current lifecycle stage
    lifecycle_stage: RwLock<PluginLifecycleStage>,

    /// Plugin configuration
    config: RwLock<PluginConfig>,

    /// Plugin dependencies
    dependencies: Vec<PluginDependency>,

    /// Plugin capabilities mapped by capability ID
    capabilities: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,

    /// Current health status
    health: RwLock<PluginHealth>,

    /// Plugin ID
    id: String,

    /// Plugin version
    version: String,
}

impl BasePlugin {
    /// Create a new base plugin
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            lifecycle_stage: RwLock::new(PluginLifecycleStage::Created),
            config: RwLock::new(PluginConfig::default()),
            dependencies: Vec::new(),
            capabilities: RwLock::new(HashMap::new()),
            health: RwLock::new(PluginHealth::Healthy),
            id: metadata.id.clone(),
            version: metadata.version.clone(),
        }
    }

    /// Add a dependency to the plugin
    pub fn with_dependency(mut self, id: impl Into<String>, version: impl Into<String>) -> Self {
        self.dependencies.push(PluginDependency::new(id, version));
        self
    }

    /// Add an optional dependency to the plugin
    pub fn with_optional_dependency(
        mut self,
        id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.dependencies
            .push(PluginDependency::optional(id, version));
        self
    }

    /// Add a capability to the plugin
    pub fn with_capability<T: std::any::Any + Send + Sync>(
        mut self,
        id: impl Into<String>,
        capability: T,
    ) -> Self {
        if let Ok(mut capabilities) = self.capabilities.write() {
            capabilities.insert(id.into(), Arc::new(capability));
        }
        self
    }

    /// Add a capability to the plugin with type ID as the ID
    pub fn with_typed_capability<T: 'static + Send + Sync>(self, capability: T) -> Self {
        let type_id = std::any::type_name::<T>();
        self.with_capability(type_id, capability)
    }

    /// Set the health status of the plugin
    pub fn set_health(&self, health: PluginHealth) {
        if let Ok(mut health_guard) = self.health.write() {
            *health_guard = health;
        }
    }

    /// Get the plugin's metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get the plugin's configuration
    pub fn config(&self) -> &PluginConfig {
        &self.config
    }

    /// Get the plugin's dependencies
    pub fn dependencies(&self) -> &[PluginDependency] {
        &self.metadata.dependencies
    }

    /// Get the plugin's capabilities
    pub fn capabilities(&self) -> &RwLock<HashMap<String, Box<dyn Any + Send + Sync>>> {
        &self.capabilities
    }

    /// Get the plugin's health status
    pub fn health(&self) -> &RwLock<PluginHealth> {
        &self.health
    }

    /// Get a capability by ID
    pub fn get_capability<T: Any + Send + Sync>(&self, capability_id: &str) -> Option<Box<T>> {
        self.capabilities()
            .read()
            .unwrap()
            .get(capability_id)
            .and_then(|cap| cap.downcast_ref::<T>().cloned())
    }
}

#[async_trait]
impl MessageHandler for BasePlugin {
    async fn handle_message(
        &mut self,
        _message: serde_json::Value,
    ) -> PluginResult<Option<serde_json::Value>> {
        // Default implementation does nothing and returns None
        Ok(None)
    }
}

#[async_trait]
impl PluginLifecycle for BasePlugin {
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
        // Store configuration
        if let Ok(mut current_config) = self.config.write() {
            *current_config = config;
        }

        // Update lifecycle stage
        if let Ok(mut stage) = self.lifecycle_stage.write() {
            *stage = PluginLifecycleStage::Initialized;
        }

        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        // Update lifecycle stage
        if let Ok(mut stage) = self.lifecycle_stage.write() {
            *stage = PluginLifecycleStage::Running;
        }

        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        // Update lifecycle stage
        if let Ok(mut stage) = self.lifecycle_stage.write() {
            *stage = PluginLifecycleStage::Stopped;
        }

        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        *self.health.read().unwrap()
    }
}

#[async_trait]
impl Plugin for BasePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn lifecycle_stage(&self) -> PluginLifecycleStage {
        *self.lifecycle_stage.read().unwrap()
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        self.dependencies.clone()
    }

    fn capabilities(&self) -> HashMap<String, Arc<dyn std::any::Any + Send + Sync>> {
        self.capabilities.read().unwrap().clone()
    }

    fn get_capability(&self, capability_id: &str) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        self.capabilities
            .read()
            .unwrap()
            .get(capability_id)
            .cloned()
    }
}

/// Builder for creating a BasePlugin
pub struct PluginBuilder {
    /// Plugin metadata
    metadata: PluginMetadata,

    /// Plugin dependencies
    dependencies: Vec<PluginDependency>,

    /// Plugin capabilities
    capabilities: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

impl PluginBuilder {
    /// Create a new plugin builder
    pub fn new(
        id: impl Into<String>,
        version: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            metadata: PluginMetadata::new(id, version, name, description, author),
            dependencies: Vec::new(),
            capabilities: HashMap::new(),
        }
    }

    /// Add a tag to the plugin metadata
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_tag(tag);
        self
    }

    /// Add multiple tags to the plugin metadata
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.metadata = self.metadata.with_tags(tags);
        self
    }

    /// Add a dependency to the plugin
    pub fn with_dependency(mut self, id: impl Into<String>, version: impl Into<String>) -> Self {
        self.dependencies.push(PluginDependency::new(id, version));
        self
    }

    /// Add an optional dependency to the plugin
    pub fn with_optional_dependency(
        mut self,
        id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.dependencies
            .push(PluginDependency::optional(id, version));
        self
    }

    /// Add a capability to the plugin
    pub fn with_capability<T: 'static + Send + Sync>(mut self, capability: T) -> Self {
        // Get type name as the capability ID
        let type_name = std::any::type_name::<T>();
        self.capabilities
            .insert(type_name.to_string(), Arc::new(capability));
        self
    }

    /// Add a capability to the plugin with a custom ID
    pub fn with_named_capability<T: 'static + Send + Sync>(
        mut self,
        id: impl Into<String>,
        capability: T,
    ) -> Self {
        self.capabilities.insert(id.into(), Arc::new(capability));
        self
    }

    /// Build the plugin
    pub fn build(self) -> Box<dyn Plugin> {
        let mut plugin = BasePlugin::new(self.metadata);
        plugin.dependencies = self.dependencies;

        // Add capabilities
        if let Ok(mut capabilities) = plugin.capabilities.write() {
            *capabilities = self.capabilities;
        }

        Box::new(plugin)
    }
}
