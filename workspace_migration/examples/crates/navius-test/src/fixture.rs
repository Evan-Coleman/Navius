use crate::error::{TestError, TestResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Internal state of the test fixture
pub struct FixtureState {
    /// Registered components by type ID
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// Resources that need cleanup
    resources: Vec<Box<dyn Resource + Send + Sync>>,
    /// Configuration
    config: FixtureConfig,
}

impl FixtureState {
    /// Create a new fixture state
    pub fn new(config: FixtureConfig) -> Self {
        Self {
            components: HashMap::new(),
            resources: Vec::new(),
            config,
        }
    }
}

/// Configuration for the test fixture
#[derive(Clone, Debug)]
pub struct FixtureConfig {
    /// Whether to automatically clean up resources
    pub auto_cleanup: bool,
    /// Whether to log fixture operations
    pub verbose_logging: bool,
}

impl Default for FixtureConfig {
    fn default() -> Self {
        Self {
            auto_cleanup: true,
            verbose_logging: false,
        }
    }
}

/// A resource that needs cleanup
pub trait Resource {
    /// Clean up the resource
    fn cleanup(&self) -> TestResult<()>;
    /// Description of the resource
    fn description(&self) -> String;
}

/// A test fixture for setting up and tearing down test resources
#[derive(Clone)]
pub struct TestFixture {
    /// The internal state of the fixture
    state: Arc<Mutex<FixtureState>>,
}

impl TestFixture {
    /// Create a new test fixture with default configuration
    pub fn new() -> TestFixtureBuilder {
        TestFixtureBuilder::new()
    }

    /// Register a component with the fixture
    pub fn register<T: Any + Send + Sync>(&self, component: T) -> TestResult<&Self> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        let type_id = TypeId::of::<T>();
        state.components.insert(type_id, Box::new(component));

        if state.config.verbose_logging {
            println!(
                "Registered component of type: {}",
                std::any::type_name::<T>()
            );
        }

        Ok(self)
    }

    /// Register a resource that needs cleanup
    pub fn register_resource<R: Resource + Send + Sync + 'static>(
        &self,
        resource: R,
    ) -> TestResult<&Self> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        if state.config.verbose_logging {
            println!("Registered resource: {}", resource.description());
        }

        state.resources.push(Box::new(resource));

        Ok(self)
    }

    /// Get a component from the fixture
    pub fn get<T: Any + Send + Sync + Clone>(&self) -> TestResult<T> {
        let state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        let type_id = TypeId::of::<T>();
        let component = state
            .components
            .get(&type_id)
            .ok_or_else(|| TestError::missing_component(std::any::type_name::<T>()))?;

        component
            .downcast_ref::<T>()
            .ok_or_else(|| {
                TestError::setup_error(format!(
                    "Component type mismatch for {}",
                    std::any::type_name::<T>()
                ))
            })
            .map(|c| c.clone())
    }

    /// Get a component by trait object
    pub fn get_as<T: ?Sized + Any>(&self) -> TestResult<Box<T>>
    where
        T: Any,
    {
        let state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        // Look for a component that implements the trait
        for (_, component) in state.components.iter() {
            if let Some(trait_obj) = component.downcast_ref::<Box<T>>() {
                return Ok(trait_obj.clone());
            }
        }

        Err(TestError::missing_component(std::any::type_name::<T>()))
    }

    /// Check if the fixture has a component of the given type
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        if let Ok(state) = self.state.lock() {
            let type_id = TypeId::of::<T>();
            state.components.contains_key(&type_id)
        } else {
            false
        }
    }

    /// Clean up all resources
    pub fn cleanup(&self) -> TestResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            TestError::teardown_error(format!("Failed to lock fixture state: {}", e))
        })?;

        let mut errors = Vec::new();

        for resource in state.resources.drain(..) {
            if let Err(e) = resource.cleanup() {
                errors.push(format!(
                    "Failed to clean up resource {}: {}",
                    resource.description(),
                    e
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TestError::teardown_error(format!(
                "Failed to clean up resources: {}",
                errors.join(", ")
            )))
        }
    }

    /// Get the configuration
    pub fn config(&self) -> TestResult<FixtureConfig> {
        let state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        Ok(state.config.clone())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Only attempt cleanup if we're the last reference
        if Arc::strong_count(&self.state) == 1 {
            if let Ok(state) = self.state.lock() {
                if state.config.auto_cleanup {
                    let _ = self.cleanup(); // Ignore errors during drop
                }
            }
        }
    }
}

/// Builder for creating test fixtures
pub struct TestFixtureBuilder {
    config: FixtureConfig,
}

impl TestFixtureBuilder {
    /// Create a new test fixture builder
    pub fn new() -> Self {
        Self {
            config: FixtureConfig::default(),
        }
    }

    /// Set whether to automatically clean up resources
    pub fn with_auto_cleanup(mut self, auto_cleanup: bool) -> Self {
        self.config.auto_cleanup = auto_cleanup;
        self
    }

    /// Set whether to log fixture operations
    pub fn with_verbose_logging(mut self, verbose_logging: bool) -> Self {
        self.config.verbose_logging = verbose_logging;
        self
    }

    /// Build the test fixture
    pub fn build(self) -> TestFixture {
        TestFixture {
            state: Arc::new(Mutex::new(FixtureState::new(self.config))),
        }
    }

    /// Add a component to the fixture
    pub fn with_component<T: Any + Send + Sync>(self, component: T) -> Self {
        let mut fixture = self.build();
        let _ = fixture.register(component);
        TestFixtureBuilder {
            config: self.config,
        }
    }

    /// Add a resource to the fixture
    pub fn with_resource<R: Resource + Send + Sync + 'static>(self, resource: R) -> Self {
        let mut fixture = self.build();
        let _ = fixture.register_resource(resource);
        TestFixtureBuilder {
            config: self.config,
        }
    }
}

impl Default for TestFixtureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Example implementation of a resource
pub struct TempDirectory {
    path: String,
}

impl TempDirectory {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl Resource for TempDirectory {
    fn cleanup(&self) -> TestResult<()> {
        // In a real implementation, this would delete the directory
        println!("Cleaning up temp directory: {}", self.path);
        Ok(())
    }

    fn description(&self) -> String {
        format!("TempDirectory({})", self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_register_and_get() {
        // Create a fixture
        let fixture = TestFixture::new().build();

        // Register a component
        fixture.register(String::from("test")).unwrap();

        // Get the component
        let value: String = fixture.get().unwrap();

        // Verify
        assert_eq!(value, "test");
    }

    #[test]
    fn test_fixture_has_component() {
        // Create a fixture
        let fixture = TestFixture::new().build();

        // Initially doesn't have the component
        assert!(!fixture.has::<i32>());

        // Register a component
        fixture.register(42).unwrap();

        // Now it has the component
        assert!(fixture.has::<i32>());
    }

    #[test]
    fn test_fixture_register_resource() {
        // Create a fixture
        let fixture = TestFixture::new().build();

        // Register a resource
        fixture
            .register_resource(TempDirectory::new("/tmp/test"))
            .unwrap();

        // Cleanup should succeed
        fixture.cleanup().unwrap();
    }

    #[test]
    fn test_fixture_builder() {
        // Create a fixture with a component
        let fixture = TestFixture::new()
            .with_verbose_logging(true)
            .with_component(42)
            .build();

        // Verify we have the component
        assert!(fixture.has::<i32>());

        // Verify configuration
        assert!(fixture.config().unwrap().verbose_logging);
    }
}
