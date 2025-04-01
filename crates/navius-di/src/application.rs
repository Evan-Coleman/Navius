// Application builder and bootstrapping for Navius Dependency Injection
//
// This module provides a comprehensive application builder pattern
// to configure and bootstrap applications with dependency injection.

use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    error::{Error, Result},
    registry::{ComponentRef, ComponentRegistry, ComponentScope},
};

/// Configuration provider trait
pub trait ConfigProvider: Send + Sync {
    /// Get a configuration value as boxed Any
    fn get_value(&self, key: &str) -> Result<Box<dyn Any + Send + Sync>>;

    /// Check if a configuration key exists
    fn has(&self, key: &str) -> bool;

    /// Get all configuration keys with a specific prefix
    fn keys_with_prefix(&self, prefix: &str) -> Vec<String>;

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Convert to Any for downcasting (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Extension trait for ConfigProvider to provide type-safe access
pub trait ConfigProviderExt: ConfigProvider {
    /// Get a configuration value by key with type checking
    fn get<T: Any + Clone + Send + Sync>(&self, key: &str) -> Result<T> {
        self.get_value(key).and_then(|value| {
            value
                .downcast_ref::<T>()
                .map(|value| value.clone())
                .ok_or_else(|| Error::ConfigBindingFailed {
                    message: format!(
                        "Cannot convert config value for key '{}' to requested type",
                        key
                    ),
                })
        })
    }
}

// Implement the extension trait for all implementors of ConfigProvider
impl<P: ?Sized + ConfigProvider> ConfigProviderExt for P {}

/// Memory-based configuration provider
pub struct MemoryConfigProvider {
    configs: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl MemoryConfigProvider {
    /// Create a new memory config provider
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Set a configuration value
    pub fn set<T: Any + Clone + Send + Sync>(&mut self, key: &str, value: T) -> &mut Self {
        self.configs.insert(key.to_string(), Box::new(value));
        self
    }
}

impl Clone for MemoryConfigProvider {
    fn clone(&self) -> Self {
        // We can't directly clone the boxed values, so we create a new empty provider
        // This is a limitation - in practice, this provider should only be cloned before values are set
        log::warn!("Cloning MemoryConfigProvider - config values will not be copied");
        Self::new()
    }
}

impl Default for MemoryConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigProvider for MemoryConfigProvider {
    fn get_value(&self, key: &str) -> Result<Box<dyn Any + Send + Sync>> {
        match self.configs.get(key) {
            Some(boxed_value) => {
                // Clone the boxed value - this is a limitation as we can't directly clone
                // a Box<dyn Any>, but we need to return a new Box with the same content
                let any_ref = boxed_value.as_ref();

                // Try to downcast to common types and clone
                if let Some(s) = any_ref.downcast_ref::<String>() {
                    return Ok(Box::new(s.clone()));
                } else if let Some(i) = any_ref.downcast_ref::<i32>() {
                    return Ok(Box::new(*i));
                } else if let Some(i) = any_ref.downcast_ref::<i64>() {
                    return Ok(Box::new(*i));
                } else if let Some(f) = any_ref.downcast_ref::<f64>() {
                    return Ok(Box::new(*f));
                } else if let Some(b) = any_ref.downcast_ref::<bool>() {
                    return Ok(Box::new(*b));
                } else if let Some(v) = any_ref.downcast_ref::<Vec<String>>() {
                    return Ok(Box::new(v.clone()));
                }

                // If we can't handle the type, return an error
                Err(Error::ConfigBindingFailed {
                    message: format!("Unable to clone config value for key '{}'", key),
                })
            }
            None => Err(Error::ConfigNotFound {
                key: key.to_string(),
            }),
        }
    }

    fn has(&self, key: &str) -> bool {
        self.configs.contains_key(key)
    }

    fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.configs
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Application plugin trait
#[async_trait]
pub trait ApplicationPlugin: Send + Sync {
    /// Initialize the plugin
    async fn initialize(
        &self,
        registry: &ComponentRegistry,
        config: &dyn ConfigProvider,
    ) -> Result<()>;

    /// Name of the plugin
    fn name(&self) -> &str;

    /// Plugin priority (lower values are initialized first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Application builder to configure and bootstrap applications
pub struct ApplicationBuilder {
    registry: ComponentRegistry,
    config_provider: Arc<dyn ConfigProvider>,
    plugins: Vec<Box<dyn ApplicationPlugin>>,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            config_provider: Arc::new(MemoryConfigProvider::new()),
            plugins: Vec::new(),
        }
    }

    /// Set the configuration provider
    pub fn with_config_provider(mut self, provider: Arc<dyn ConfigProvider>) -> Self {
        self.config_provider = provider;
        self
    }

    /// Add a component to the registry
    pub fn with_component<T: Any + Send + Sync>(self, component: T) -> Self {
        self.registry.register(component).unwrap_or_else(|e| {
            log::warn!("Failed to register component: {}", e);
        });
        self
    }

    /// Add a component with a qualifier
    pub fn with_component_and_qualifier<T: Any + Send + Sync>(
        self,
        component: T,
        qualifier: &str,
    ) -> Self {
        self.registry
            .register_with_qualifier(component, qualifier)
            .unwrap_or_else(|e| {
                log::warn!(
                    "Failed to register component with qualifier '{}': {}",
                    qualifier,
                    e
                );
            });
        self
    }

    /// Add a component factory
    pub fn with_factory<T, F>(self, factory: F, scope: ComponentScope) -> Self
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.registry.register_with_factory::<T, F>(factory, scope);
        self
    }

    /// Add a component factory with a qualifier
    pub fn with_factory_and_qualifier<T, F>(
        self,
        factory: F,
        scope: ComponentScope,
        qualifier: &str,
    ) -> Self
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.registry
            .register_with_factory_and_qualifier::<T, F>(factory, scope, qualifier);
        self
    }

    /// Add a plugin
    pub fn with_plugin<P: ApplicationPlugin + 'static>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// Set a configuration value
    pub fn with_config<T: Any + Clone + Send + Sync>(mut self, key: &str, value: T) -> Self {
        if let Some(provider) = Arc::get_mut(&mut self.config_provider) {
            if let Some(memory_provider) =
                provider.as_any_mut().downcast_mut::<MemoryConfigProvider>()
            {
                memory_provider.set(key, value);
            } else {
                log::warn!("Cannot set config value directly on non-memory config provider");
            }
        } else {
            log::warn!("Cannot get mutable reference to config provider");
        }
        self
    }

    /// Register the config provider as a component
    pub fn with_config_as_component(self) -> Self {
        // Clone the Arc rather than trying to clone the inner provider
        let provider = self.config_provider.clone();

        // Register the provider
        self.registry
            .register_with_qualifier(provider, "configProvider")
            .unwrap_or_else(|e| {
                log::warn!("Failed to register config provider: {}", e);
            });

        self
    }

    /// Build and initialize the application
    pub async fn build(mut self) -> Result<Application> {
        // Sort plugins by priority
        self.plugins.sort_by_key(|p| p.priority());

        // Initialize all plugins
        for plugin in &self.plugins {
            log::info!("Initializing plugin: {}", plugin.name());
            plugin
                .initialize(&self.registry, self.config_provider.as_ref())
                .await?;
        }

        // Create the application
        Ok(Application {
            registry: self.registry,
            config_provider: Box::new(ConfigProviderClone(self.config_provider)),
        })
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper struct for Arc<dyn ConfigProvider> to simplify conversion to Box
struct ConfigProviderClone(Arc<dyn ConfigProvider>);

impl ConfigProvider for ConfigProviderClone {
    fn get_value(&self, key: &str) -> Result<Box<dyn Any + Send + Sync>> {
        self.0.get_value(key)
    }

    fn has(&self, key: &str) -> bool {
        self.0.has(key)
    }

    fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.0.keys_with_prefix(prefix)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Application instance with initialized components
pub struct Application {
    registry: ComponentRegistry,
    config_provider: Box<dyn ConfigProvider>,
}

impl Application {
    /// Create a new application builder
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    /// Get a component from the registry
    pub fn get<T: Any + Send + Sync>(&self) -> Result<ComponentRef<T>> {
        self.registry.get::<T>()
    }

    /// Get a component by qualifier
    pub fn get_by_qualifier<T: Any + Send + Sync>(
        &self,
        qualifier: &str,
    ) -> Result<ComponentRef<T>> {
        self.registry.get_by_qualifier::<T>(qualifier)
    }

    /// Get a configuration value
    pub fn config<T: Any + Clone + Send + Sync>(&self, key: &str) -> Result<T> {
        self.config_provider.get::<T>(key)
    }

    /// Get all configuration keys with a specific prefix
    pub fn config_keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.config_provider.keys_with_prefix(prefix)
    }

    /// Shutdown the application
    pub fn shutdown(&self) -> Result<()> {
        self.registry.shutdown()
    }

    /// Shutdown the application asynchronously
    pub async fn shutdown_async(&self) -> Result<()> {
        self.registry.shutdown_async().await
    }
}
