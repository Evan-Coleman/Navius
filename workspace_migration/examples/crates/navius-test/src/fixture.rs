use crate::error::{TestError, TestResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
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

/// A test fixture that provides dependencies for tests
#[derive(Debug)]
pub struct TestFixture {
    /// A registry of components for the fixture
    components: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,

    /// Resources that need cleanup when the fixture is dropped
    resources: Mutex<Vec<Box<dyn Resource + Send + Sync>>>,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            resources: Mutex::new(Vec::new()),
        }
    }

    /// Register a component with the fixture
    pub fn register<T: Any + Send + Sync>(&self, component: T) -> TestResult<()> {
        let mut components = self.components.write().map_err(|_| {
            TestError::FixtureError("Failed to acquire write lock on components".to_string())
        })?;

        let type_id = TypeId::of::<T>();
        components.insert(type_id, Box::new(component));

        Ok(())
    }

    /// Get a component by type
    pub fn get_component<T: Any + ?Sized>(&self) -> TestResult<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let components = self.components.lock().expect("Failed to lock components");

        let component_any = components.get(&type_id).ok_or_else(|| {
            TestError::FixtureError(format!("Component of type ID {:?} not found", type_id))
        })?;

        // Clone the Arc before returning it
        let component_arc = Arc::clone(component_any);

        component_arc.downcast::<T>().map_err(|_| {
            TestError::FixtureError(format!(
                "Component for type ID {:?} is not of the expected type",
                type_id
            ))
        })
    }

    /// Add a resource to the fixture
    pub fn add_resource<R: Resource + 'static>(&self, resource: R) {
        let mut resources = self.resources.lock().expect("Failed to lock resources");
        resources.push(Box::new(resource));
    }

    /// Clean up all resources
    pub fn cleanup(&self) -> TestResult<()> {
        let mut resources = self.resources.lock().map_err(|_| {
            TestError::TeardownError("Failed to acquire lock on resources".to_string())
        })?;

        let mut errors = Vec::new();

        for resource in resources.iter_mut() {
            if let Err(e) = resource.cleanup() {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TestError::TeardownError(format!(
                "Failed to clean up resources: {:?}",
                errors
            )))
        }
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// A test context for running tests
#[derive(Debug)]
pub struct TestContext {
    /// The test fixture
    fixture: Arc<TestFixture>,

    /// The name of the test
    name: String,

    /// Whether the test is running
    running: bool,
}

impl TestContext {
    /// Create a new test context
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            fixture: Arc::new(TestFixture::new()),
            name: name.into(),
            running: false,
        }
    }

    /// Get the name of the test
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the test fixture
    pub fn fixture(&self) -> &Arc<TestFixture> {
        &self.fixture
    }

    /// Mark the test as running
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Mark the test as not running
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if the test is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

/// A builder for creating test fixtures
#[derive(Debug)]
pub struct TestFixtureBuilder {
    /// The fixture being built
    fixture: TestFixture,
}

impl TestFixtureBuilder {
    /// Create a new test fixture builder
    pub fn new() -> Self {
        Self {
            fixture: TestFixture::new(),
        }
    }

    /// Add a component to the fixture
    pub fn with_component<T: Any + Send + Sync>(self, component: T) -> TestResult<Self> {
        self.fixture.register(component)?;
        Ok(self)
    }

    /// Add a resource to the fixture builder
    pub fn with_resource<R: Resource + 'static>(self, resource: R) -> Self {
        self.fixture.add_resource(resource);
        self
    }

    /// Build the fixture
    pub fn build(self) -> TestFixture {
        self.fixture
    }
}

impl Default for TestFixtureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A file resource that needs to be cleaned up
#[derive(Debug)]
pub struct FileResource {
    /// The path to the file
    path: String,

    /// Whether to delete the file on cleanup
    delete_on_cleanup: bool,
}

impl FileResource {
    /// Create a new file resource
    pub fn new<S: Into<String>>(path: S, delete_on_cleanup: bool) -> Self {
        Self {
            path: path.into(),
            delete_on_cleanup,
        }
    }

    /// Get the path to the file
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Resource for FileResource {
    fn cleanup(&mut self) -> TestResult<()> {
        if self.delete_on_cleanup {
            if let Err(e) = std::fs::remove_file(&self.path) {
                return Err(TestError::TeardownError(format!(
                    "Failed to remove file {}: {}",
                    self.path, e
                )));
            }
        }

        Ok(())
    }
}

/// A directory resource that needs to be cleaned up
#[derive(Debug)]
pub struct DirectoryResource {
    /// The path to the directory
    path: String,

    /// Whether to delete the directory on cleanup
    delete_on_cleanup: bool,
}

impl DirectoryResource {
    /// Create a new directory resource
    pub fn new<S: Into<String>>(path: S, delete_on_cleanup: bool) -> Self {
        Self {
            path: path.into(),
            delete_on_cleanup,
        }
    }

    /// Get the path to the directory
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Resource for DirectoryResource {
    fn cleanup(&mut self) -> TestResult<()> {
        if self.delete_on_cleanup {
            if let Err(e) = std::fs::remove_dir_all(&self.path) {
                return Err(TestError::TeardownError(format!(
                    "Failed to remove directory {}: {}",
                    self.path, e
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_register_and_get() {
        let fixture = TestFixture::new();

        // Register a component
        fixture.register(42i32).unwrap();

        // Get the component
        let value = fixture.get_component::<i32>().unwrap();
        assert_eq!(*value, 42);

        // Register another component
        fixture.register("hello".to_string()).unwrap();

        // Get the string component
        let string = fixture.get_component::<String>().unwrap();
        assert_eq!(string, "hello");
    }

    #[test]
    fn test_fixture_get_missing_component() {
        let fixture = TestFixture::new();

        // Try to get a component that doesn't exist
        let result = fixture.get_component::<i32>();
        assert!(result.is_err());
    }

    #[test]
    fn test_fixture_builder() {
        let fixture = TestFixtureBuilder::new()
            .with_component(42i32)
            .unwrap()
            .with_component("hello".to_string())
            .unwrap()
            .build();

        // Get the components
        let value = fixture.get_component::<i32>().unwrap();
        assert_eq!(*value, 42);

        let string = fixture.get_component::<String>().unwrap();
        assert_eq!(string, "hello");
    }

    #[test]
    fn test_file_resource() {
        let temp_file = std::env::temp_dir().join("test_file.txt");
        let path = temp_file.to_string_lossy().to_string();

        // Create the file
        std::fs::write(&path, b"test").unwrap();

        // Create a file resource
        let mut resource = FileResource::new(&path, true);

        // Check that the file exists
        assert!(std::fs::metadata(&path).is_ok());

        // Clean up the resource
        resource.cleanup().unwrap();

        // Check that the file was deleted
        assert!(std::fs::metadata(&path).is_err());
    }

    #[test]
    fn test_directory_resource() {
        let temp_dir = std::env::temp_dir().join("test_dir");
        let path = temp_dir.to_string_lossy().to_string();

        // Create the directory
        std::fs::create_dir_all(&path).unwrap();

        // Create a directory resource
        let mut resource = DirectoryResource::new(&path, true);

        // Check that the directory exists
        assert!(std::fs::metadata(&path).is_ok());

        // Clean up the resource
        resource.cleanup().unwrap();

        // Check that the directory was deleted
        assert!(std::fs::metadata(&path).is_err());
    }
}
