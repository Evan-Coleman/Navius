//! Application bootstrapping with component registry integration.
//!
//! This module provides application initialization utilities that integrate with
//! the component registry for dependency injection.

use std::{any::Any, sync::Arc};

use crate::config::Config;
use crate::error::{Error, Result};

use super::component::{ComponentRef, ComponentRegistry, ComponentScope};

/// Application builder with component registry
pub struct ApplicationBuilder {
    /// Component registry for the application
    registry: ComponentRegistry,
    /// Application configuration
    config: Config,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            config: Config::default(),
        }
    }

    /// Create a new application builder with a configuration
    pub fn with_config(config: Config) -> Self {
        Self {
            registry: ComponentRegistry::new(),
            config,
        }
    }

    /// Get a reference to the component registry
    pub fn registry(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Add a component to the registry
    pub fn add_component<T: Any + Send + Sync>(&mut self, component: T) -> &mut Self {
        self.registry.register(component);
        self
    }

    /// Add a component factory with scope
    pub fn add_factory<T, F>(&mut self, factory: F, scope: ComponentScope) -> &mut Self
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.registry.register_with_factory(factory, scope);
        self
    }

    /// Add a singleton component factory
    pub fn add_singleton<T, F>(&mut self, factory: F) -> &mut Self
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.registry
            .register_with_factory(factory, ComponentScope::Singleton);
        self
    }

    /// Add a prototype component factory
    pub fn add_prototype<T, F>(&mut self, factory: F) -> &mut Self
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.registry
            .register_with_factory(factory, ComponentScope::Prototype);
        self
    }

    /// Get a component from the registry
    pub fn get<T: Any + Send + Sync>(&mut self) -> Result<ComponentRef<T>> {
        self.registry.get::<T>()
    }

    /// Check if a component exists in the registry
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        self.registry.has::<T>()
    }

    /// Build the application
    pub fn build(self) -> Application {
        Application {
            registry: Arc::new(std::sync::Mutex::new(self.registry)),
            config: self.config,
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Application with component registry
pub struct Application {
    /// Component registry for the application
    registry: Arc<std::sync::Mutex<ComponentRegistry>>,
    /// Application configuration
    config: Config,
}

impl Application {
    /// Create a new application builder
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    /// Get a reference to the component registry
    pub fn registry(&self) -> Arc<std::sync::Mutex<ComponentRegistry>> {
        self.registry.clone()
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a component from the registry
    pub fn get<T: Any + Send + Sync>(&self) -> Result<ComponentRef<T>> {
        let mut registry = self.registry.lock().map_err(|e| {
            Error::new(&format!(
                "Failed to acquire lock on component registry: {}",
                e
            ))
        })?;
        registry.get::<T>()
    }

    /// Check if a component exists in the registry
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        if let Ok(registry) = self.registry.lock() {
            registry.has::<T>()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestComponent {
        value: String,
    }

    #[derive(Debug, Clone)]
    struct ConfigComponent {
        config_value: String,
    }

    #[test]
    fn builder_pattern() {
        let app = Application::builder()
            .add_component(TestComponent {
                value: "test".to_string(),
            })
            .build();

        let component = app.get::<TestComponent>();
        assert!(component.is_ok());
        assert_eq!(component.unwrap().value, "test");
    }

    #[test]
    fn singleton_components() {
        let app = Application::builder()
            .add_singleton::<TestComponent, _>(|| TestComponent {
                value: "singleton".to_string(),
            })
            .build();

        let component1 = app.get::<TestComponent>().unwrap();
        let component2 = app.get::<TestComponent>().unwrap();

        assert_eq!(component1.value, "singleton");
        assert_eq!(component2.value, "singleton");
    }

    #[test]
    fn config_integration() {
        let mut config = Config::default();
        config.set("app.name", "test-app").unwrap();

        let app = Application::builder()
            .with_config(config)
            .add_singleton::<ConfigComponent, _>(|| {
                let config_value = "config-test".to_string();
                ConfigComponent { config_value }
            })
            .build();

        assert_eq!(app.config().get::<String>("app.name").unwrap(), "test-app");

        let component = app.get::<ConfigComponent>().unwrap();
        assert_eq!(component.config_value, "config-test");
    }
}
