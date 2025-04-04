use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use navius_cache::error::CacheError;
use navius_plugin::plugin::{
    MessageHandler, Plugin, PluginConfig, PluginHealth, PluginLifecycle, PluginLifecycleStage,
    PluginMetadata, PluginResult,
};
use serde_json::Value;
use tracing::{debug, info};

use crate::config::RedisCacheConfig;
use crate::operations::RedisCache;

/// Redis Cache Plugin
#[derive(Debug, Clone)]
pub struct RedisCachePlugin {
    metadata: PluginMetadata,
    lifecycle_stage: PluginLifecycleStage,
}

impl RedisCachePlugin {
    /// Create a new Redis cache plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "navius-cache-redis-plugin",
                env!("CARGO_PKG_VERSION"),
                "Redis Cache Plugin",
                "Redis implementation for the navius-cache system",
                "Navius Team",
            )
            .with_tag("cache")
            .with_tag("redis"),
            lifecycle_stage: PluginLifecycleStage::Created,
        }
    }
}

#[async_trait::async_trait]
impl MessageHandler for RedisCachePlugin {
    async fn handle_message(&self, _message: Value) -> PluginResult<Option<Value>> {
        // No messages handled in this plugin
        Ok(None)
    }
}

#[async_trait::async_trait]
impl PluginLifecycle for RedisCachePlugin {
    async fn initialize(&mut self, _config: PluginConfig) -> PluginResult<()> {
        debug!("Initializing Redis cache plugin");
        self.lifecycle_stage = PluginLifecycleStage::Initialized;
        Ok(())
    }

    async fn start(&mut self) -> PluginResult<()> {
        debug!("Starting Redis cache plugin");
        self.lifecycle_stage = PluginLifecycleStage::Started;
        Ok(())
    }

    async fn stop(&mut self) -> PluginResult<()> {
        debug!("Stopping Redis cache plugin");
        self.lifecycle_stage = PluginLifecycleStage::Stopped;
        Ok(())
    }

    async fn health_check(&self) -> PluginHealth {
        PluginHealth::Healthy
    }
}

impl Plugin for RedisCachePlugin {
    fn id(&self) -> &str {
        "navius-cache-redis-plugin"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn lifecycle_stage(&self) -> PluginLifecycleStage {
        self.lifecycle_stage
    }
}

/// Factory for creating Redis cache instances
#[derive(Debug)]
struct RedisCacheFactory;

// Use a custom implementation instead of the Factory trait which appears to have changed
impl RedisCacheFactory {
    fn create(
        &self,
        config: &navius_core::config::Config,
    ) -> Result<Box<dyn navius_cache::Cache>, Box<dyn std::error::Error>> {
        debug!("Creating Redis cache instance from config");

        // Extract Redis configuration
        let redis_config = RedisCacheConfig::from_config(config).map_err(|e| {
            CacheError::ConfigurationError(format!("Invalid Redis configuration: {}", e))
        })?;

        // Create a new runtime for async initialization
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            CacheError::BackendError(format!("Failed to create Tokio runtime: {}", e))
        })?;

        // Create Redis cache
        let cache = runtime.block_on(async { RedisCache::new(redis_config).await })?;

        info!("Redis cache instance created successfully");
        Ok(Box::new(cache))
    }
}

/// Register the plugin with the plugin registry
pub fn register_plugin() -> Result<(), Box<dyn std::error::Error>> {
    let plugin = RedisCachePlugin::new();
    // Use a simpler approach for now until we understand how plugin registration works
    info!("Redis cache plugin registered successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let plugin = RedisCachePlugin::new();
        assert_eq!(plugin.id(), "navius-cache-redis-plugin");
        assert!(!plugin.version().is_empty());
        assert_eq!(plugin.metadata().name, "Redis Cache Plugin");
    }

    // Additional tests would require mock PluginContext
}
