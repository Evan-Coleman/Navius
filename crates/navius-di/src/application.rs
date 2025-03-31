// Application builder and bootstrapping for Navius Dependency Injection
//
// This module provides a comprehensive application builder pattern
// to configure and bootstrap applications with dependency injection.

use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    error::{Error, Result},
    registry::{
        ComponentFactory, ComponentRef, ComponentRegistry, ComponentScope, DynComponentRef,
    },
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
impl<P: ConfigProvider> ConfigProviderExt for P {}

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

impl Default for MemoryConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigProvider for MemoryConfigProvider {
    fn get_value(&self, key: &str) -> Result<Box<dyn Any + Send + Sync>> {
        self.configs
            .get(key)
            .map(|value| value.clone())
            .ok_or_else(|| Error::ConfigNotFound {
                key: key.to_string(),
            })
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
        if let Some(provider) = self
            .config_provider
            .as_mut()
            .downcast_mut::<MemoryConfigProvider>()
        {
            provider.set(key, value);
        } else {
            log::warn!("Cannot set config value directly on non-memory config provider");
        }
        self
    }

    /// Register the config provider as a component
    pub fn with_config_as_component(self) -> Self {
        // Create a new reference to the config provider
        let provider = Arc::new(self.config_provider.as_ref().clone_box());

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
            config_provider: self.config_provider,
        })
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Clone box method for ConfigProvider
trait CloneBox {
    fn clone_box(&self) -> Box<dyn ConfigProvider>;
}

impl<T: ConfigProvider + Clone + 'static> CloneBox for T {
    fn clone_box(&self) -> Box<dyn ConfigProvider> {
        Box::new(self.clone())
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
