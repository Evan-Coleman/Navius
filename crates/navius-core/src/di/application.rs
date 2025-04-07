//! Application bootstrapping with component registry integration.
//!
//! This module provides application initialization utilities that integrate with
//! the component registry for dependency injection.

use std::any::Any;

use crate::config::Config;
use crate::error::{Error, Result};

use super::component::{ComponentRef, ComponentScope, DynComponentRef, LifecyclePhase};
use crate::di::registry::{ComponentRegistry, InMemoryComponentRegistry};
use std::sync::{Arc, Mutex};

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
    registry: Mutex<InMemoryComponentRegistry>,
    /// Application configuration
    config: Config,
    /// Application environment
    environment: Environment,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            registry: Mutex::new(InMemoryComponentRegistry::new()),
            config: Config::default(),
            environment: Environment::default(),
        }
    }

    /// Create a new application builder with a configuration
    pub fn with_config(config: Config) -> Self {
        Self {
            registry: Mutex::new(InMemoryComponentRegistry::new()),
            config,
            environment: Environment::default(),
        }
    }

    /// Set the environment for the application
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Get a mutable reference to the component registry mutex.
    pub fn registry(&mut self) -> &mut Mutex<InMemoryComponentRegistry> {
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
    pub fn add_component(&mut self, component: DynComponentRef) -> Result<&mut Self> {
        self.registry
            .lock()
            .map_err(|_| Error::internal("Mutex poisoned"))?
            .register(component)?;
        Ok(self)
    }

    /// Add a component factory with scope
    pub fn add_factory<T, F>(&mut self, factory: F, scope: ComponentScope) -> Result<&mut Self>
    where
        T: Any + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        // Create the component using the factory first
        let component = factory();
        // Wrap it in DynComponentRef
        let component_ref = DynComponentRef::new(component);
        // Register the DynComponentRef
        self.add_component(component_ref)?;
        Ok(self)
        // FIXME: This doesn't handle scope correctly (always registers instance)
        // Needs proper factory registration support if scope != Singleton
    }

    /// Add a singleton component factory
    pub fn add_singleton<T, F>(&mut self, factory: F) -> Result<&mut Self>
    where
        T: Any + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        // Create instance and register it directly for Singleton
        let component = factory();
        let component_ref = DynComponentRef::new(component);
        self.add_component(component_ref)
    }

    /// Add a prototype component factory
    pub fn add_prototype<T, F>(&mut self, factory: F) -> Result<&mut Self>
    where
        T: Any + Send + Sync + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        // FIXME: Prototype requires registering the factory itself, not an instance.
        // This currently behaves like add_singleton.
        println!("Warning: add_prototype currently registers a singleton instance.");
        let component = factory();
        let component_ref = DynComponentRef::new(component);
        self.add_component(component_ref)
    }

    /// Build the application
    pub fn build(self) -> Application {
        Application {
            registry: Arc::new(self.registry),
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

/// Represents the Navius application context, holding the component registry.
#[derive(Debug, Clone)]
pub struct Application {
    /// Component registry for the application
    registry: Arc<Mutex<InMemoryComponentRegistry>>,
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

    /// Get a reference to the component registry Arc.
    pub fn registry(&self) -> &Arc<Mutex<InMemoryComponentRegistry>> {
        &self.registry
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
    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_name = std::any::type_name::<T>();
        self.registry
            .lock()
            .ok()?
            .get_by_type_name(type_name)
            // downcast now returns Option<Arc<T>>, so no .map() needed
            .and_then(|dyn_ref| dyn_ref.clone().downcast::<T>())
    }

    /// Check if a component exists in the registry
    pub fn has<T: Any + Send + Sync + 'static>(&self) -> bool {
        let type_name = std::any::type_name::<T>();
        self.registry
            .lock()
            .ok()
            .map_or(false, |reg| reg.get_by_type_name(type_name).is_some())
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
        // Use add_singleton which now works correctly (ish)
        let app = Application::builder()
            .add_singleton(|| TestComponent::new("test"))
            .unwrap()
            .build();

        let component = app.get::<TestComponent>(); // Use sync get
        assert!(component.is_some());
        assert_eq!(component.unwrap().value, "test");
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
            .add_singleton(|| TestComponent::new("singleton"))
            .unwrap()
            .build();

        let component1 = app.get::<TestComponent>().unwrap();
        let component2 = app.get::<TestComponent>().unwrap();

        assert_eq!(component1.value, "singleton");
        assert_eq!(component2.value, "singleton");
        // Check Arc pointer equality
        assert!(Arc::ptr_eq(&component1, &component2));
    }

    #[test]
    fn config_integration() {
        let mut config = Config::default();
        config.set("app.name", "test-app").unwrap();

        let app = Application::builder()
            .with_config(config) // with_config needs fixing if it returns Self
            .add_singleton(|| {
                let config_value = "config-test".to_string();
                ConfigComponent { config_value }
            })
            .unwrap()
            .build();

        assert_eq!(app.config().get::<String>("app.name").unwrap(), "test-app");

        let component = app.get::<ConfigComponent>().unwrap();
        assert_eq!(component.config_value, "config-test");
    }
}
