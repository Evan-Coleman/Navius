//! Application bootstrapping with component registry integration.
//!
//! This module provides application initialization utilities that integrate with
//! the component registry for dependency injection.

use std::{any::Any, sync::Arc};

use crate::config::Config;
use crate::error::{Error, Result};

use super::component::{ComponentRef, ComponentRegistry, ComponentScope, LifecyclePhase};

/// Environment for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Development environment
    Development,
    /// Testing environment
    Testing,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

impl Environment {
    /// Get the name of the environment
    pub fn name(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Testing => "testing",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }

    /// Parse an environment name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "development" | "dev" => Some(Self::Development),
            "testing" | "test" => Some(Self::Testing),
            "staging" => Some(Self::Staging),
            "production" | "prod" => Some(Self::Production),
            _ => None,
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Application builder with component registry
pub struct ApplicationBuilder {
    /// Component registry for the application
    registry: ComponentRegistry,
    /// Application configuration
    config: Config,
    /// Application environment
    environment: Environment,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            config: Config::default(),
            environment: Environment::default(),
        }
    }

    /// Create a new application builder with a configuration
    pub fn with_config(config: Config) -> Self {
        Self {
            registry: ComponentRegistry::new(),
            config,
            environment: Environment::default(),
        }
    }

    /// Set the environment for the application
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Get a reference to the component registry
    pub fn registry(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the environment
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// Add a component to the registry
    pub fn add_component<T: Any + Send + Sync>(&mut self, component: T) -> Result<&mut Self> {
        self.registry.register(component)?;
        Ok(self)
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

    /// Get a component from the registry with async initialization
    pub async fn get_async<T: Any + Send + Sync>(&mut self) -> Result<ComponentRef<T>> {
        self.registry.get_async::<T>().await
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
            environment: self.environment,
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
    /// Application environment
    environment: Environment,
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

    /// Get the application environment
    pub fn environment(&self) -> Environment {
        self.environment
    }

    /// Get a component from the registry
    pub fn get<T: Any + Send + Sync>(&self) -> Result<ComponentRef<T>> {
        let mut registry = self.registry.lock().map_err(|e| {
            Error::internal(&format!(
                "Failed to acquire lock on component registry: {}",
                e
            ))
        })?;
        registry.get::<T>()
    }

    /// Get a component from the registry with async initialization
    pub async fn get_async<T: Any + Send + Sync>(&self) -> Result<ComponentRef<T>> {
        let mut registry = self.registry.lock().map_err(|e| {
            Error::internal(&format!(
                "Failed to acquire lock on component registry: {}",
                e
            ))
        })?;
        registry.get_async::<T>().await
    }

    /// Check if a component exists in the registry
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        if let Ok(registry) = self.registry.lock() {
            registry.has::<T>()
        } else {
            false
        }
    }

    /// Register a component with the application
    pub fn register_component<T: 'static + Send + Sync>(&self, component: T) -> Result<()> {
        let mut registry = self.registry.lock().map_err(|e| {
            Error::internal(&format!(
                "Failed to acquire lock on component registry: {}",
                e
            ))
        })?;

        registry.register(component);
        Ok(())
    }

    /// Initialize the application and start the lifecycles of all registered components
    pub fn initialize(&self) -> Result<()> {
        let registry = self.registry.lock().map_err(|e| {
            Error::internal(&format!(
                "Failed to acquire lock on component registry: {}",
                e
            ))
        })?;

        // Initialize all components in the registry
        for component in registry.values() {
            component.initialize()?;
        }

        Ok(())
    }

    /// Shut down all registered components in the application
    pub fn shutdown(&self) -> Result<()> {
        let registry = self.registry.lock().map_err(|e| {
            Error::internal(&format!(
                "Failed to acquire lock on component registry during shutdown: {}",
                e
            ))
        })?;

        // Shut down all components in reverse initialization order
        for component in registry.values().rev() {
            component.shutdown()?;
        }

        Ok(())
    }

    /// Shut down all registered components asynchronously
    pub async fn shutdown_async(&self) -> Result<()> {
        let registry = self.registry.lock().map_err(|e| {
            Error::internal(&format!(
                "Failed to acquire lock on component registry during async shutdown: {}",
                e
            ))
        })?;

        // Shut down all components asynchronously in reverse initialization order
        for component in registry.values().rev() {
            component.shutdown_async().await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::component::Lifecycle;
    use super::*;

    #[derive(Debug, Clone)]
    struct TestComponent {
        value: String,
        initialized: bool,
        destroyed: bool,
    }

    impl TestComponent {
        fn new(value: &str) -> Self {
            Self {
                value: value.to_string(),
                initialized: false,
                destroyed: false,
            }
        }
    }

    impl Lifecycle for TestComponent {
        fn on_initialize(&self) -> Result<()> {
            println!("Initializing TestComponent: {}", self.value);
            let mut this = self as *const Self as *mut Self;
            unsafe {
                (*this).initialized = true;
            }
            Ok(())
        }

        fn on_destroy(&self) -> Result<()> {
            println!("Destroying TestComponent: {}", self.value);
            let mut this = self as *const Self as *mut Self;
            unsafe {
                (*this).destroyed = true;
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct ConfigComponent {
        config_value: String,
    }

    #[test]
    fn builder_pattern() {
        let app = Application::builder()
            .add_component(TestComponent::new("test"))
            .unwrap()
            .build();

        let component = app.get::<TestComponent>();
        assert!(component.is_ok());
        assert_eq!(component.unwrap().value, "test");
    }

    #[test]
    fn lifecycle_hooks() {
        let app = Application::builder()
            .add_singleton::<TestComponent, _>(|| TestComponent::new("lifecycle"))
            .build();

        let component = app.get::<TestComponent>().unwrap();
        assert_eq!(component.value, "lifecycle");
        assert!(component.initialized);

        app.shutdown().unwrap();
        // Note: we can't test destroyed flag here as the component is dropped after shutdown
    }

    #[test]
    fn environment_configuration() {
        let app = Application::builder()
            .with_environment(Environment::Production)
            .build();

        assert_eq!(app.environment(), Environment::Production);
        assert_eq!(app.environment().name(), "production");
    }

    #[test]
    fn singleton_components() {
        let app = Application::builder()
            .add_singleton::<TestComponent, _>(|| TestComponent::new("singleton"))
            .build();

        let component1 = app.get::<TestComponent>().unwrap();
        let component2 = app.get::<TestComponent>().unwrap();

        assert_eq!(component1.value, "singleton");
        assert_eq!(component2.value, "singleton");
        assert!(std::ptr::eq(&*component1, &*component2));
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

    #[tokio::test]
    async fn async_lifecycle() {
        use super::super::component::AsyncLifecycle;

        #[derive(Debug)]
        struct AsyncComponent {
            initialized: bool,
            destroyed: bool,
        }

        #[async_trait::async_trait]
        impl AsyncLifecycle for AsyncComponent {
            async fn on_initialize_async(&self) -> Result<()> {
                println!("Async initializing");
                let mut this = self as *const Self as *mut Self;
                unsafe {
                    (*this).initialized = true;
                }
                Ok(())
            }

            async fn on_destroy_async(&self) -> Result<()> {
                println!("Async destroying");
                let mut this = self as *const Self as *mut Self;
                unsafe {
                    (*this).destroyed = true;
                }
                Ok(())
            }
        }

        let mut app_builder = Application::builder();
        app_builder
            .registry()
            .register(AsyncComponent {
                initialized: false,
                destroyed: false,
            })
            .unwrap();

        let app = app_builder.build();

        let component = app.get_async::<AsyncComponent>().await.unwrap();
        assert!(component.initialized);

        app.shutdown_async().await.unwrap();
        // Again, we can't check destroyed flag as the component is dropped
    }
}
