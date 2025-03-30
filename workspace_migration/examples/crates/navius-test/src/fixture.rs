use crate::error::{TestError, TestResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// A test fixture that manages resources and components for tests
#[derive(Clone)]
pub struct TestFixture {
    /// The internal state of the fixture
    state: Arc<Mutex<FixtureState>>,
}

/// The internal state of a test fixture
struct FixtureState {
    /// Map of registered components by type ID
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,

    /// Temporary directories created for the test
    temp_dirs: Vec<TempDir>,

    /// Configuration values
    config: HashMap<String, String>,

    /// Flag indicating whether the fixture is already torn down
    torn_down: bool,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new() -> TestFixtureBuilder {
        TestFixtureBuilder::new()
    }

    /// Register a component with the fixture
    pub fn register<T: Any + Send + Sync>(&self, component: T) -> TestResult<()> {
        let type_id = TypeId::of::<T>();
        let mut state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        state.components.insert(type_id, Box::new(component));
        Ok(())
    }

    /// Get a component from the fixture
    pub fn get<T: Any + Send + Sync>(&self) -> TestResult<T> {
        let type_id = TypeId::of::<T>();
        let state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        state
            .components
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .map(|component| component.clone())
            .ok_or_else(|| TestError::missing_component(std::any::type_name::<T>()))
    }

    /// Check if a component is registered
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let state = self.state.lock().unwrap_or_else(|e| {
            // In case of error, log and return an empty state
            eprintln!("Failed to lock fixture state: {}", e);
            Default::default()
        });

        state.components.contains_key(&type_id)
    }

    /// Create a temporary directory
    pub fn create_temp_dir(&self, prefix: &str) -> TestResult<PathBuf> {
        let temp_dir = TempDir::new(prefix).map_err(|e| {
            TestError::resource_allocation_error(format!("Failed to create temp dir: {}", e))
        })?;

        let path = temp_dir.path().to_path_buf();

        let mut state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        state.temp_dirs.push(temp_dir);

        Ok(path)
    }

    /// Set a configuration value
    pub fn set_config(&self, key: &str, value: &str) -> TestResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        state.config.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> TestResult<String> {
        let state = self
            .state
            .lock()
            .map_err(|e| TestError::setup_error(format!("Failed to lock fixture state: {}", e)))?;

        state
            .config
            .get(key)
            .map(|s| s.clone())
            .ok_or_else(|| TestError::missing_component(format!("Config key '{}'", key)))
    }

    /// Tear down the fixture
    pub fn tear_down(&self) -> TestResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            TestError::teardown_error(format!("Failed to lock fixture state: {}", e))
        })?;

        if state.torn_down {
            return Ok(());
        }

        // Mark as torn down first to prevent repeated teardown attempts
        state.torn_down = true;

        // Clear all components
        state.components.clear();

        // Temp dirs are cleaned up automatically when dropped
        state.temp_dirs.clear();

        Ok(())
    }
}

impl fmt::Debug for TestFixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.state.lock() {
            Ok(state) => {
                let component_count = state.components.len();
                let temp_dir_count = state.temp_dirs.len();
                let config_count = state.config.len();

                f.debug_struct("TestFixture")
                    .field("components", &format!("{} registered", component_count))
                    .field("temp_dirs", &format!("{} directories", temp_dir_count))
                    .field("config", &format!("{} values", config_count))
                    .finish()
            }
            Err(_) => write!(f, "TestFixture(locked)"),
        }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Attempt to tear down the fixture if it's the last reference
        if Arc::strong_count(&self.state) == 1 {
            let _ = self.tear_down();
        }
    }
}

impl Default for FixtureState {
    fn default() -> Self {
        Self {
            components: HashMap::new(),
            temp_dirs: Vec::new(),
            config: HashMap::new(),
            torn_down: false,
        }
    }
}

/// Builder for creating test fixtures
pub struct TestFixtureBuilder {
    /// Components to register
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,

    /// Configuration values
    config: HashMap<String, String>,
}

impl TestFixtureBuilder {
    /// Create a new test fixture builder
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            config: HashMap::new(),
        }
    }

    /// Register a component with the fixture
    pub fn with_component<T: Any + Send + Sync>(mut self, component: T) -> Self {
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, Box::new(component));
        self
    }

    /// Set a configuration value
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the test fixture
    pub fn build(self) -> TestFixture {
        let mut state = FixtureState::default();

        state.components = self.components;
        state.config = self.config;

        TestFixture {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

/// Extension to TestFixtureBuilder for registering typed components
impl<T: Any + Send + Sync> ComponentRegistration<T> for TestFixtureBuilder {
    fn register(self, component: T) -> Self {
        self.with_component(component)
    }
}

/// Trait for registering components in a type-safe way
pub trait ComponentRegistration<T: Any + Send + Sync> {
    /// Register a component
    fn register(self, component: T) -> Self;
}

/// Wrapper for a component that can be registered with a test fixture
pub struct Component<T: Any + Send + Sync> {
    /// The component value
    value: T,

    /// Marker type
    _marker: PhantomData<T>,
}

impl<T: Any + Send + Sync> Component<T> {
    /// Create a new component wrapper
    pub fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Unwrap the component value
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Any + Send + Sync + Clone> Clone for Component<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Any + Send + Sync + fmt::Debug> fmt::Debug for Component<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Component")
            .field("value", &self.value)
            .finish()
    }
}
