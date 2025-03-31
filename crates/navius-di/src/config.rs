// Configuration module for Navius Dependency Injection
//
// This module provides configuration binding and injection support
// with prefix-based configuration organization.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use serde::de::DeserializeOwned;

use crate::{
    application::ConfigProvider,
    error::{Error, Result},
};

/// Configuration binding abstraction
pub trait ConfigBinding {
    /// Bind configuration to a type
    fn bind<T: Any + DeserializeOwned + Send + Sync>(&self, prefix: &str) -> Result<T>;
}

/// Configuration binder using a ConfigProvider
pub struct Binder<'a> {
    provider: &'a dyn ConfigProvider,
}

impl<'a> Binder<'a> {
    /// Create a new binder
    pub fn new(provider: &'a dyn ConfigProvider) -> Self {
        Self { provider }
    }
}

impl<'a> ConfigBinding for Binder<'a> {
    fn bind<T: Any + DeserializeOwned + Send + Sync>(&self, prefix: &str) -> Result<T> {
        // Get all keys with the given prefix
        let keys = self.provider.keys_with_prefix(prefix);

        // Build a HashMap of key-value pairs
        let mut config_map = HashMap::new();
        for key in keys {
            if let Ok(value) = self.provider.get::<String>(&key) {
                // Remove the prefix from the key
                let trimmed_key = if key.starts_with(prefix) {
                    if key.len() > prefix.len() + 1 {
                        // +1 to skip the dot or separator
                        key[prefix.len() + 1..].to_string()
                    } else {
                        // Empty key, skip
                        continue;
                    }
                } else {
                    key.clone()
                };

                config_map.insert(trimmed_key, value);
            }
        }

        // Convert the map to the target type using serde
        serde_json::to_value(config_map)
            .map_err(|e| {
                Error::ConfigBindingFailed(format!("Failed to serialize config map: {}", e))
            })
            .and_then(|json| {
                serde_json::from_value::<T>(json).map_err(|e| {
                    Error::ConfigBindingFailed(format!(
                        "Failed to deserialize to target type: {}",
                        e
                    ))
                })
            })
    }
}

/// Configuration registry for caching and managing bound configurations
pub struct ConfigRegistry {
    provider: Arc<dyn ConfigProvider>,
    bound_configs: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl ConfigRegistry {
    /// Create a new config registry
    pub fn new(provider: Arc<dyn ConfigProvider>) -> Self {
        Self {
            provider,
            bound_configs: RwLock::new(HashMap::new()),
        }
    }

    /// Get or bind a configuration for a type
    pub fn get_or_bind<T: Any + DeserializeOwned + Clone + Send + Sync>(
        &self,
        prefix: &str,
    ) -> Result<T> {
        let type_id = TypeId::of::<T>();

        // Check if already bound
        {
            let bound = self.bound_configs.read().unwrap();
            if let Some(config) = bound.get(&type_id) {
                if let Some(config) = config.downcast_ref::<T>() {
                    return Ok(config.clone());
                }
            }
        }

        // Not found, bind it
        let binder = Binder::new(self.provider.as_ref());
        let config = binder.bind::<T>(prefix)?;

        // Cache the result
        {
            let mut bound_configs = self.bound_configs.write().unwrap();
            bound_configs.insert(type_id, Box::new(config.clone()));
        }

        Ok(config)
    }

    /// Get all known configuration prefixes
    pub fn get_all_prefixes(&self) -> Vec<String> {
        // Extract unique prefixes from keys
        let mut prefixes = Vec::new();
        let keys = self.provider.keys_with_prefix("");

        for key in keys {
            if let Some(prefix) = key.split('.').next() {
                if !prefixes.contains(&prefix.to_string()) {
                    prefixes.push(prefix.to_string());
                }
            }
        }

        prefixes
    }
}

/// Configuration reference wrapper for injecting configurations
#[derive(Debug, Clone)]
pub struct ConfigRef<T: Clone> {
    inner: Arc<T>,
}

impl<T: Clone> ConfigRef<T> {
    /// Create a new config reference
    pub fn new(config: T) -> Self {
        Self {
            inner: Arc::new(config),
        }
    }

    /// Get the inner config value
    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.inner).unwrap_or_else(|arc| (*arc).clone())
    }
}

impl<T: Clone> std::ops::Deref for ConfigRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Configuration prefix attribute for defining config prefixes
pub trait ConfigPrefix {
    /// Get the configuration prefix
    fn prefix() -> &'static str;
}

/// Trait for creating a configuration from a registry
pub trait Configurable: Sized + Clone + Send + Sync + 'static {
    /// Create configuration from registry
    fn create(registry: &ConfigRegistry) -> Result<Self>;
}

/// Automatically implement Configurable for types that have a ConfigPrefix
impl<T> Configurable for T
where
    T: ConfigPrefix + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn create(registry: &ConfigRegistry) -> Result<Self> {
        registry.get_or_bind::<T>(T::prefix())
    }
}

/// Create a configuration reference from a registry
pub fn config_ref<T: Configurable>(registry: &ConfigRegistry) -> Result<ConfigRef<T>> {
    let config = T::create(registry)?;
    Ok(ConfigRef::new(config))
}
