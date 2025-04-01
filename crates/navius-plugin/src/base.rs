use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::error::{PluginHealth, PluginResult};
use crate::plugin::{
    MessageHandler, Plugin, PluginConfig, PluginDependency, PluginLifecycle, PluginLifecycleStage,
    PluginMetadata,
};

/// A simple base implementation of the Plugin trait
#[derive(Debug)]
pub struct BasePlugin {
    metadata: PluginMetadata,
    stage: PluginLifecycleStage,
    capabilities: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

impl BasePlugin {
    /// Create a new base plugin
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            stage: PluginLifecycleStage::Created,
            capabilities: HashMap::new(),
        }
    }

    /// Add a capability to the plugin
    pub fn add_capability(
        &mut self,
        id: impl Into<String>,
        capability: Arc<dyn std::any::Any + Send + Sync>,
    ) {
        self.capabilities.insert(id.into(), capability);
    }

    /// Get the plugin configuration
    pub fn config(&self) -> PluginConfig {
        PluginConfig::default()
    }

    /// Get the plugin capabilities
    pub fn capabilities(&self) -> HashMap<String, Arc<dyn std::any::Any + Send + Sync>> {
        self.capabilities.clone()
    }
}

#[async_trait]
impl MessageHandler for BasePlugin {
    async fn handle_message(&self, _message: Value) -> PluginResult<Option<Value>> {
        Ok(None)
    }
}

#[async_trait]
impl PluginLifecycle for BasePlugin {
    async fn initialize(&mut self, _config: PluginConfig) -> PluginResult<()> {
        self.stage = PluginLifecycleStage::Initialized;
        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        self.stage = PluginLifecycleStage::Started;
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        self.stage = PluginLifecycleStage::Stopped;
        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        PluginHealth::Healthy
    }
}

impl Plugin for BasePlugin {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn version(&self) -> &str {
        &self.metadata.version
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn lifecycle_stage(&self) -> PluginLifecycleStage {
        self.stage
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        Vec::new()
    }

    fn get_capability(&self, capability_id: &str) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        self.capabilities.get(capability_id).cloned()
    }
}

/// Builder for creating plugins
#[derive(Debug)]
pub struct PluginBuilder {
    metadata: PluginMetadata,
    capabilities: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

impl PluginBuilder {
    /// Create a new plugin builder
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        let name_str = name.into();
        Self {
            metadata: PluginMetadata::new(
                name_str.clone(), // id is same as name
                version,
                name_str,
                description,
                author,
            ),
            capabilities: HashMap::new(),
        }
    }

    /// Add a capability to the plugin
    pub fn with_capability(
        mut self,
        id: impl Into<String>,
        capability: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        self.capabilities.insert(id.into(), capability);
        self
    }

    /// Add a tag to the plugin
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_tag(tag);
        self
    }

    /// Add multiple tags to the plugin
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.metadata = self.metadata.with_tags(tags);
        self
    }

    /// Build the plugin
    pub fn build(self) -> BasePlugin {
        let mut plugin = BasePlugin::new(self.metadata);
        for (id, capability) in self.capabilities {
            plugin.add_capability(id, capability);
        }
        plugin
    }
}
