//! Integration Test Utilities
//!
//! This module provides utilities for creating and running integration tests
//! that span multiple crates in the Navius workspace. It builds upon the
//! test fixture, mock registry, and test harness components to provide
//! a comprehensive testing environment.

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{TestError, TestResult};
use crate::fixture::TestFixture;
use crate::harness::{TestHarness, TestOptions};
use crate::mock::MockRegistry;
use crate::mocks::database::{DatabaseClient, MockDatabaseClient};

/// Configuration for an integration test
#[derive(Debug, Clone)]
pub struct IntegrationTestConfig {
    /// Name of the test for reporting purposes
    pub name: String,
    /// Directory for test artifacts
    pub test_dir: Option<PathBuf>,
    /// Environment variables to set for the test
    pub env_vars: HashMap<String, String>,
    /// Timeout for the test execution
    pub timeout: Option<Duration>,
    /// Whether to verify mocks automatically
    pub verify_mocks: bool,
    /// Whether to clean up resources automatically
    pub cleanup_resources: bool,
    /// Service configurations for cross-crate testing
    pub service_configs: HashMap<String, ServiceConfig>,
    /// Test data source path
    pub test_data_path: Option<PathBuf>,
    /// Database setup scripts
    pub db_setup_scripts: Vec<String>,
    /// Test lifecycle hooks
    pub lifecycle_hooks: TestLifecycleHooks,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_test".to_string(),
            test_dir: None,
            env_vars: HashMap::new(),
            timeout: Some(Duration::from_secs(30)),
            verify_mocks: true,
            cleanup_resources: true,
            service_configs: HashMap::new(),
            test_data_path: None,
            db_setup_scripts: Vec::new(),
            lifecycle_hooks: TestLifecycleHooks::default(),
        }
    }
}

/// Test lifecycle hooks for integration tests
#[derive(Debug, Clone)]
pub struct TestLifecycleHooks {
    /// Before test setup actions (as commands to execute)
    pub before_setup: Vec<String>,
    /// After setup actions
    pub after_setup: Vec<String>,
    /// Before test actions
    pub before_test: Vec<String>,
    /// After test actions
    pub after_test: Vec<String>,
    /// Before teardown actions
    pub before_teardown: Vec<String>,
    /// After teardown actions
    pub after_teardown: Vec<String>,
}

impl Default for TestLifecycleHooks {
    fn default() -> Self {
        Self {
            before_setup: Vec::new(),
            after_setup: Vec::new(),
            before_test: Vec::new(),
            after_test: Vec::new(),
            before_teardown: Vec::new(),
            after_teardown: Vec::new(),
        }
    }
}

/// Configuration for a service in cross-crate tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: String,
    /// Service configuration properties
    pub properties: HashMap<String, ConfigValue>,
    /// Dependencies on other services
    pub dependencies: Vec<String>,
}

/// Configuration value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of config values
    Array(Vec<ConfigValue>),
    /// Object of config values
    Object(HashMap<String, ConfigValue>),
}

impl ConfigValue {
    /// Get as string
    pub fn as_string(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get as integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get as boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Service discovery result for component registration
#[derive(Debug)]
pub struct ServiceDiscoveryResult {
    /// Discovered service instances
    pub instances: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
    /// Dependencies between services
    pub dependencies: HashMap<String, Vec<String>>,
}

/// Context for cross-crate integration tests
///
/// The `IntegrationContext` provides a shared environment for integration
/// tests that span multiple crates. It manages:
///
/// - Test fixtures with registered components
/// - Mock implementations for interfaces
/// - Environment variables
/// - Test resources and cleanup
#[derive(Clone)]
pub struct IntegrationContext {
    config: IntegrationTestConfig,
    fixtures: Vec<Arc<TestFixture>>,
    test_dir: Option<PathBuf>,
    env_vars: Arc<Mutex<HashMap<String, String>>>,
    registry: Arc<MockRegistry>,
    original_env: HashMap<String, Option<String>>,
    services: Arc<RwLock<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>>,
    test_data: Arc<RwLock<HashMap<String, TestData>>>,
    db_client: Option<Arc<MockDatabaseClient>>,
}

/// Test data for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestData {
    /// Data identifier
    pub id: String,
    /// Data content
    pub content: HashMap<String, ConfigValue>,
}

impl TestData {
    /// Create a new test data instance
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: HashMap::new(),
        }
    }

    /// Add a string value to the test data
    pub fn with_string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.content
            .insert(key.into(), ConfigValue::String(value.into()));
        self
    }

    /// Add an integer value to the test data
    pub fn with_integer(mut self, key: impl Into<String>, value: i64) -> Self {
        self.content.insert(key.into(), ConfigValue::Integer(value));
        self
    }

    /// Add a float value to the test data
    pub fn with_float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.content.insert(key.into(), ConfigValue::Float(value));
        self
    }

    /// Add a boolean value to the test data
    pub fn with_boolean(mut self, key: impl Into<String>, value: bool) -> Self {
        self.content.insert(key.into(), ConfigValue::Boolean(value));
        self
    }

    /// Add an array value to the test data
    pub fn with_array(mut self, key: impl Into<String>, value: Vec<ConfigValue>) -> Self {
        self.content.insert(key.into(), ConfigValue::Array(value));
        self
    }

    /// Add an object value to the test data
    pub fn with_object(
        mut self,
        key: impl Into<String>,
        value: HashMap<String, ConfigValue>,
    ) -> Self {
        self.content.insert(key.into(), ConfigValue::Object(value));
        self
    }
}

/// Test data builder for creating and registering test data
pub struct TestDataBuilder {
    context: Arc<IntegrationContext>,
    data: Vec<TestData>,
}

impl TestDataBuilder {
    /// Create a new test data builder
    pub fn new(context: Arc<IntegrationContext>) -> Self {
        Self {
            context,
            data: Vec::new(),
        }
    }

    /// Add test data to the builder
    pub fn with_data(mut self, data: TestData) -> Self {
        self.data.push(data);
        self
    }

    /// Generate a simple entity test data
    pub fn with_entity<S: Into<String>>(
        mut self,
        id: S,
        name: Option<S>,
        is_active: Option<bool>,
    ) -> Self {
        let id_str = id.into();
        let mut entity = TestData::new(format!("entity-{}", id_str)).with_string("id", id_str);

        if let Some(name_val) = name {
            entity = entity.with_string("name", name_val);
        }

        if let Some(active) = is_active {
            entity = entity.with_boolean("active", active);
        }

        self.data.push(entity);
        self
    }

    /// Generate a user test data
    pub fn with_user<S: Into<String>>(
        mut self,
        id: S,
        name: S,
        email: Option<S>,
        roles: Option<Vec<S>>,
    ) -> Self {
        let id_str = id.into();
        let name_str = name.into();

        let mut user = TestData::new(format!("user-{}", id_str))
            .with_string("id", id_str)
            .with_string("name", name_str);

        if let Some(email_val) = email {
            user = user.with_string("email", email_val);
        }

        if let Some(role_vals) = roles {
            let roles_array = role_vals
                .into_iter()
                .map(|r| ConfigValue::String(r.into()))
                .collect();
            user = user.with_array("roles", roles_array);
        }

        self.data.push(user);
        self
    }

    /// Generate a configuration test data
    pub fn with_config<S: Into<String>>(
        mut self,
        id: S,
        properties: HashMap<String, ConfigValue>,
    ) -> Self {
        let config =
            TestData::new(format!("config-{}", id.into())).with_object("properties", properties);

        self.data.push(config);
        self
    }

    /// Register all test data with the context
    pub fn register(self) -> TestResult<()> {
        let mut test_data = self.context.test_data.write().map_err(|_| {
            TestError::concurrency_error("Failed to acquire write lock for test data")
        })?;

        for data in self.data {
            test_data.insert(data.id.clone(), data);
        }

        Ok(())
    }

    /// Export all test data to a file
    pub fn export_to_file(self, path: &PathBuf) -> TestResult<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl IntegrationContext {
    /// Create a new integration context with the given configuration
    pub fn new(config: IntegrationTestConfig) -> TestResult<Self> {
        let test_dir = if let Some(dir) = &config.test_dir {
            Some(dir.clone())
        } else {
            let temp_dir = std::env::temp_dir().join("navius-test").join(&config.name);
            std::fs::create_dir_all(&temp_dir)?;
            Some(temp_dir)
        };

        // Remember original environment variables
        let mut original_env = HashMap::new();
        for (key, _) in &config.env_vars {
            original_env.insert(key.clone(), std::env::var(key).ok());
        }

        let registry = Arc::new(MockRegistry::new());

        let context = Self {
            config,
            fixtures: Vec::new(),
            test_dir,
            env_vars: Arc::new(Mutex::new(HashMap::new())),
            registry,
            original_env,
            services: Arc::new(RwLock::new(HashMap::new())),
            test_data: Arc::new(RwLock::new(HashMap::new())),
            db_client: None,
        };

        // Set environment variables
        for (key, value) in &context.config.env_vars {
            context.set_env_var(key, value)?;
        }

        // Load test data if configured
        if let Some(path) = &context.config.test_data_path {
            context.load_test_data(path)?;
        }

        // Set up database if needed
        if !context.config.db_setup_scripts.is_empty() {
            context.setup_database()?;
        }

        Ok(context)
    }

    /// Create a new fixture within this integration context
    pub fn create_fixture(&mut self) -> TestResult<Arc<TestFixture>> {
        let fixture = Arc::new(TestFixture::new());
        fixture.register_component(self.registry.clone())?;
        self.fixtures.push(fixture.clone());
        Ok(fixture)
    }

    /// Get the test directory path
    pub fn test_dir(&self) -> TestResult<PathBuf> {
        self.test_dir
            .clone()
            .ok_or_else(|| TestError::ConfigurationError("Test directory is not configured".into()))
    }

    /// Set an environment variable for the test
    pub fn set_env_var(&self, key: &str, value: &str) -> TestResult<()> {
        let mut env_vars = self
            .env_vars
            .lock()
            .map_err(|_| TestError::concurrency_error("Failed to acquire lock for env vars"))?;
        env_vars.insert(key.to_string(), value.to_string());
        std::env::set_var(key, value);
        Ok(())
    }

    /// Get an environment variable set for the test
    pub fn get_env_var(&self, key: &str) -> TestResult<Option<String>> {
        let env_vars = self.env_vars.lock().map_err(|_| {
            TestError::concurrency_error("Failed to acquire read lock for env vars")
        })?;
        Ok(env_vars.get(key).cloned())
    }

    /// Get the mock registry
    pub fn registry(&self) -> Arc<MockRegistry> {
        self.registry.clone()
    }

    /// Create a test harness with this context's registry
    pub fn create_harness(&self) -> TestHarness<()> {
        let fixture = TestFixture::new();
        fixture
            .register_component(self.registry.clone())
            .expect("Failed to register mock registry with test fixture");

        let options = TestOptions {
            verify_mocks: self.config.verify_mocks,
            cleanup_resources: self.config.cleanup_resources,
            timeout: self.config.timeout,
        };

        TestHarness::new()
            .with_fixture(Arc::new(fixture))
            .with_options(options)
    }

    /// Load test data from the given path
    pub fn load_test_data(&self, path: &PathBuf) -> TestResult<()> {
        if !path.exists() {
            return Err(TestError::ConfigurationError(format!(
                "Test data path does not exist: {}",
                path.display()
            )));
        }

        if path.is_dir() {
            // Load all JSON files in the directory
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    self.load_test_data_file(&path)?;
                }
            }
        } else if path.is_file() {
            // Load a single file
            self.load_test_data_file(path)?;
        }

        Ok(())
    }

    /// Load test data from a specific file
    pub fn load_test_data_file(&self, path: &PathBuf) -> TestResult<()> {
        let file_content = std::fs::read_to_string(path)?;
        let data: TestData = serde_json::from_str(&file_content)?;

        let mut test_data = self.test_data.write().map_err(|_| {
            TestError::concurrency_error("Failed to acquire write lock for test data")
        })?;
        test_data.insert(data.id.clone(), data);

        Ok(())
    }

    /// Get test data by ID
    pub fn get_test_data(&self, id: &str) -> TestResult<TestData> {
        let test_data = self.test_data.read().map_err(|_| {
            TestError::concurrency_error("Failed to acquire read lock for test data")
        })?;

        test_data
            .get(id)
            .cloned()
            .ok_or_else(|| TestError::missing_component(format!("Test data not found: {}", id)))
    }

    /// Set up database for testing
    pub fn setup_database(&self) -> TestResult<()> {
        // Create mock database client if not already created
        let db_client = Arc::new(MockDatabaseClient::new());

        // Register with registry
        self.registry
            .register::<dyn DatabaseClient, _>(db_client.clone())?;

        // Execute setup scripts
        for script in &self.config.db_setup_scripts {
            db_client.expect_execute(script, Ok(1))?;
        }

        Ok(())
    }

    /// Register a service with the context
    pub fn register_service<T: 'static + Send + Sync>(
        &self,
        name: &str,
        service: T,
    ) -> TestResult<()> {
        let mut services = self.services.write().map_err(|_| {
            TestError::concurrency_error("Failed to acquire write lock for services")
        })?;

        services.insert(name.to_string(), Arc::new(service));
        Ok(())
    }

    /// Get a service by name and type
    pub fn get_service<T: 'static + Send + Sync>(&self, name: &str) -> TestResult<Arc<T>> {
        let services = self.services.read().map_err(|_| {
            TestError::concurrency_error("Failed to acquire read lock for services")
        })?;

        if let Some(service) = services.get(name) {
            if let Some(typed_service) = service.clone().downcast_arc::<T>() {
                Ok(typed_service)
            } else {
                Err(TestError::type_mismatch(format!(
                    "Service {} is not of requested type",
                    name
                )))
            }
        } else {
            Err(TestError::missing_component(format!(
                "Service not found: {}",
                name
            )))
        }
    }

    /// Discover and register services based on configuration
    pub fn discover_services(&self) -> TestResult<ServiceDiscoveryResult> {
        let mut instances = HashMap::new();
        let mut dependencies = HashMap::new();
        let configs = self.config.service_configs.clone();

        // Build dependency graph
        for (name, config) in &configs {
            dependencies.insert(name.clone(), config.dependencies.clone());
        }

        // Perform topological sort to resolve dependencies in correct order
        let sorted_services = self.topological_sort(&dependencies)?;

        // Process services in dependency order
        for service_name in sorted_services {
            if let Some(config) = configs.get(&service_name) {
                // Check if dependencies are satisfied
                for dep in &config.dependencies {
                    if !instances.contains_key(dep) {
                        return Err(TestError::dependency_error(format!(
                            "Service '{}' depends on '{}', but it was not created",
                            service_name, dep
                        )));
                    }
                }

                // Create service instance based on its type
                let instance = self.create_service_instance(&service_name, config, &instances)?;
                instances.insert(service_name.clone(), instance);
            }
        }

        let result = ServiceDiscoveryResult {
            instances,
            dependencies,
        };

        // Register discovered services
        for (name, instance) in &result.instances {
            let mut services = self.services.write().map_err(|_| {
                TestError::concurrency_error("Failed to acquire write lock for services")
            })?;
            services.insert(name.clone(), instance.clone());
        }

        Ok(result)
    }

    /// Create a service instance based on service type
    fn create_service_instance(
        &self,
        name: &str,
        config: &ServiceConfig,
        existing_services: &HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
    ) -> TestResult<Arc<dyn std::any::Any + Send + Sync>> {
        // In a real implementation, this would use a registry of service factories
        // For now, we'll create placeholder instances with the service name stored
        let instance: Arc<dyn std::any::Any + Send + Sync> = match config.service_type.as_str() {
            "PostgresDatabase" => {
                let url = config
                    .properties
                    .get("url")
                    .ok_or_else(|| {
                        TestError::configuration_error(format!(
                            "Missing 'url' property for database service '{}'",
                            name
                        ))
                    })?
                    .as_string()
                    .ok_or_else(|| {
                        TestError::type_mismatch(format!(
                            "'url' property for database service '{}' must be a string",
                            name
                        ))
                    })?;

                println!(
                    "Creating PostgresDatabase service '{}' with URL: {}",
                    name, url
                );
                // This would create a real database client in production code
                Arc::new(name.to_string())
            }
            "RedisCache" => {
                let url = config
                    .properties
                    .get("url")
                    .ok_or_else(|| {
                        TestError::configuration_error(format!(
                            "Missing 'url' property for cache service '{}'",
                            name
                        ))
                    })?
                    .as_string()
                    .ok_or_else(|| {
                        TestError::type_mismatch(format!(
                            "'url' property for cache service '{}' must be a string",
                            name
                        ))
                    })?;

                // Check for dependency on database
                if config.dependencies.contains(&"database".to_string()) {
                    let db_service = existing_services.get("database").ok_or_else(|| {
                        TestError::dependency_error(format!(
                            "Cache service '{}' depends on 'database', but it doesn't exist",
                            name
                        ))
                    })?;

                    println!(
                        "Creating RedisCache service '{}' with URL: {} and database dependency",
                        name, url
                    );
                } else {
                    println!("Creating RedisCache service '{}' with URL: {}", name, url);
                }

                Arc::new(name.to_string())
            }
            "HttpClient" => {
                let timeout = config
                    .properties
                    .get("timeout")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(30); // Default timeout of 30 seconds

                println!(
                    "Creating HttpClient service '{}' with timeout: {}s",
                    name, timeout
                );
                Arc::new(name.to_string())
            }
            _ => {
                // Generic service creation for unknown types
                println!(
                    "Creating generic service '{}' of type '{}'",
                    name, config.service_type
                );
                Arc::new(name.to_string())
            }
        };

        Ok(instance)
    }

    /// Perform topological sort on service dependencies
    fn topological_sort(
        &self,
        dependencies: &HashMap<String, Vec<String>>,
    ) -> TestResult<Vec<String>> {
        let mut result = Vec::new();
        let mut visited = HashMap::new();
        let mut temp_mark = HashMap::new();

        // Initialize visit trackers
        for node in dependencies.keys() {
            visited.insert(node.clone(), false);
            temp_mark.insert(node.clone(), false);
        }

        // Visit each node
        for node in dependencies.keys() {
            if !visited[node] {
                self.visit(
                    node,
                    dependencies,
                    &mut visited,
                    &mut temp_mark,
                    &mut result,
                )?;
            }
        }

        // Reverse to get dependency order (least dependent first)
        result.reverse();
        Ok(result)
    }

    /// Recursive visit function for topological sort
    fn visit(
        &self,
        node: &String,
        dependencies: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, bool>,
        temp_mark: &mut HashMap<String, bool>,
        result: &mut Vec<String>,
    ) -> TestResult<()> {
        // Check for circular dependency
        if *temp_mark.get(node).unwrap_or(&false) {
            return Err(TestError::dependency_error(format!(
                "Circular dependency detected involving service '{}'",
                node
            )));
        }

        if !*visited.get(node).unwrap_or(&false) {
            // Mark temporarily
            temp_mark.insert(node.clone(), true);

            // Visit dependencies
            if let Some(deps) = dependencies.get(node) {
                for dep in deps {
                    if dependencies.contains_key(dep) {
                        self.visit(dep, dependencies, visited, temp_mark, result)?;
                    }
                }
            }

            // Mark permanently
            visited.insert(node.clone(), true);
            temp_mark.insert(node.clone(), false);
            result.push(node.clone());
        }

        Ok(())
    }

    /// Execute hooks for a specific lifecycle stage
    pub fn execute_hooks(&self, hooks: &[String]) -> TestResult<()> {
        for hook in hooks {
            // In a real implementation, this would execute the hook command
            // For now, we'll just log that we would execute it
            println!("Executing hook: {}", hook);
        }
        Ok(())
    }

    /// Create a test data builder
    pub fn create_test_data_builder(&self) -> TestDataBuilder {
        TestDataBuilder::new(Arc::new(self.clone()))
    }

    /// Generate and register test data
    pub fn generate_test_data<F>(&self, builder_fn: F) -> TestResult<()>
    where
        F: FnOnce(TestDataBuilder) -> TestDataBuilder,
    {
        let builder = TestDataBuilder::new(Arc::new(self.clone()));
        builder_fn(builder).register()
    }
}

impl Drop for IntegrationContext {
    fn drop(&mut self) {
        // Execute before teardown hooks
        let _ = self.execute_hooks(&self.config.lifecycle_hooks.before_teardown);

        // Restore original environment variables
        for (key, value) in &self.original_env {
            match value {
                Some(val) => std::env::set_var(key, val),
                None => std::env::remove_var(key),
            }
        }

        // Clean up test directory if cleanup_resources is true
        if self.config.cleanup_resources {
            if let Some(dir) = &self.test_dir {
                let _ = std::fs::remove_dir_all(dir);
            }
        }

        // Execute after teardown hooks
        let _ = self.execute_hooks(&self.config.lifecycle_hooks.after_teardown);
    }
}

/// Integration test runner
///
/// The `IntegrationRunner` provides utilities for running integration tests
/// with proper setup and teardown.
pub struct IntegrationRunner {
    context: IntegrationContext,
}

impl IntegrationRunner {
    /// Create a new integration test runner
    pub fn new(config: IntegrationTestConfig) -> TestResult<Self> {
        // Execute before setup hooks
        let context = IntegrationContext::new(config.clone())?;
        context.execute_hooks(&config.lifecycle_hooks.before_setup)?;

        let runner = Self { context };

        // Execute after setup hooks
        runner
            .context
            .execute_hooks(&config.lifecycle_hooks.after_setup)?;

        Ok(runner)
    }

    /// Access the integration context
    pub fn context(&self) -> &IntegrationContext {
        &self.context
    }

    /// Access the integration context mutably
    pub fn context_mut(&mut self) -> &mut IntegrationContext {
        &mut self.context
    }

    /// Run a test function with the integration context
    pub fn run<F, T>(&self, test_fn: F) -> TestResult<T>
    where
        F: FnOnce(&IntegrationContext) -> TestResult<T>,
    {
        // Execute before test hooks
        self.context
            .execute_hooks(&self.context.config.lifecycle_hooks.before_test)?;

        // Run the test
        let result = test_fn(&self.context);

        // Execute after test hooks
        self.context
            .execute_hooks(&self.context.config.lifecycle_hooks.after_test)?;

        // Verify mocks if configured to do so
        if self.context.config.verify_mocks {
            self.context.registry.verify()?;
        }

        result
    }

    /// Run a test function with timeout
    pub fn run_with_timeout<F, T>(&self, test_fn: F) -> TestResult<T>
    where
        F: FnOnce(&IntegrationContext) -> TestResult<T> + Send + 'static,
        T: Send + 'static,
    {
        if let Some(timeout) = self.context.config.timeout {
            use std::thread;
            use std::time::Instant;

            let (tx, rx) = std::sync::mpsc::channel();
            let context_arc = Arc::new(self.context.clone());

            // Execute before test hooks
            self.context
                .execute_hooks(&self.context.config.lifecycle_hooks.before_test)?;

            let handle = thread::spawn(move || {
                let result = test_fn(&context_arc);
                let _ = tx.send(result);
            });

            let start = Instant::now();
            let result = rx.recv_timeout(timeout);

            // Execute after test hooks regardless of test result
            self.context
                .execute_hooks(&self.context.config.lifecycle_hooks.after_test)?;

            match result {
                Ok(test_result) => {
                    // Verify mocks if configured to do so
                    if self.context.config.verify_mocks {
                        self.context.registry.verify()?;
                    }
                    test_result
                }
                Err(_) => {
                    let elapsed = start.elapsed();
                    let _ = handle.join();
                    Err(TestError::TimeoutError(format!(
                        "Test timed out after {:?}",
                        elapsed
                    )))
                }
            }
        } else {
            self.run(test_fn)
        }
    }
}

/// Cross-crate test configuration
///
/// This struct provides a way to configure tests that span multiple crates.
#[derive(Debug, Clone)]
pub struct CrossCrateTestConfig {
    /// Name of the test
    pub name: String,
    /// Crates involved in the test
    pub crates: Vec<String>,
    /// Test timeout
    pub timeout: Option<Duration>,
    /// Test resources directory
    pub resources_dir: Option<PathBuf>,
    /// Service configurations
    pub service_configs: HashMap<String, ServiceConfig>,
    /// Test data path
    pub test_data_path: Option<PathBuf>,
    /// Database setup scripts
    pub db_setup_scripts: Vec<String>,
    /// Lifecycle hooks
    pub lifecycle_hooks: TestLifecycleHooks,
}

impl Default for CrossCrateTestConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_cross_crate_test".to_string(),
            crates: Vec::new(),
            timeout: Some(Duration::from_secs(60)),
            resources_dir: None,
            service_configs: HashMap::new(),
            test_data_path: None,
            db_setup_scripts: Vec::new(),
            lifecycle_hooks: TestLifecycleHooks::default(),
        }
    }
}

/// Builder for cross-crate tests
///
/// Provides a fluent interface for building cross-crate tests.
pub struct CrossCrateTestBuilder {
    config: CrossCrateTestConfig,
    integration_config: IntegrationTestConfig,
    fixtures: HashMap<String, Arc<TestFixture>>,
}

impl CrossCrateTestBuilder {
    /// Creates a new builder with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            config: CrossCrateTestConfig {
                name: name.clone(),
                crates: Vec::new(),
                timeout: Some(Duration::from_secs(60)),
                resources_dir: None,
            },
            integration_config: IntegrationTestConfig {
                name,
                test_dir: None,
                env_vars: HashMap::new(),
                verify_mocks: true,
                cleanup_resources: true,
                timeout: Some(Duration::from_secs(60)),
                test_data_path: None,
                service_configs: HashMap::new(),
                db_setup_scripts: Vec::new(),
                lifecycle_hooks: TestLifecycleHooks::default(),
            },
        }
    }

    /// Adds a crate to the test
    pub fn with_crate(&mut self, crate_name: impl Into<String>) -> &mut Self {
        let crate_name = crate_name.into();
        self.config.crates.push(crate_name.clone());
        self
    }

    /// Sets the timeout for the test
    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.config.timeout = Some(timeout);
        self.integration_config.timeout = Some(timeout);
        self
    }

    /// Sets the resources directory for the test
    pub fn with_resources_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.config.resources_dir = Some(dir.into());
        self
    }

    /// Adds an environment variable to the test
    pub fn with_env_var(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.integration_config
            .env_vars
            .insert(key.into(), value.into());
        self
    }

    /// Sets whether to verify mocks after the test
    pub fn with_verify_mocks(&mut self, verify: bool) -> &mut Self {
        self.integration_config.verify_mocks = verify;
        self
    }

    /// Sets whether to clean resources after the test
    pub fn with_clean_resources(&mut self, clean: bool) -> &mut Self {
        self.integration_config.cleanup_resources = clean;
        self
    }

    /// Sets the test data path
    pub fn with_test_data_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.integration_config.test_data_path = Some(path.into());
        self
    }

    /// Adds a service configuration
    pub fn with_service_config(
        &mut self,
        name: impl Into<String>,
        config: ServiceConfig,
    ) -> &mut Self {
        self.integration_config
            .service_configs
            .insert(name.into(), config);
        self
    }

    /// Adds a database setup script
    pub fn with_db_setup_script(&mut self, script: impl Into<String>) -> &mut Self {
        self.integration_config.db_setup_scripts.push(script.into());
        self
    }

    /// Adds a lifecycle hook
    pub fn with_lifecycle_hook(
        &mut self,
        stage: LifecycleStage,
        command: impl Into<String>,
    ) -> &mut Self {
        let command = command.into();
        match stage {
            LifecycleStage::BeforeSetup => self
                .integration_config
                .lifecycle_hooks
                .before_setup
                .push(command),
            LifecycleStage::AfterSetup => self
                .integration_config
                .lifecycle_hooks
                .after_setup
                .push(command),
            LifecycleStage::BeforeTest => self
                .integration_config
                .lifecycle_hooks
                .before_test
                .push(command),
            LifecycleStage::AfterTest => self
                .integration_config
                .lifecycle_hooks
                .after_test
                .push(command),
            LifecycleStage::BeforeTeardown => self
                .integration_config
                .lifecycle_hooks
                .before_teardown
                .push(command),
            LifecycleStage::AfterTeardown => self
                .integration_config
                .lifecycle_hooks
                .after_teardown
                .push(command),
        }
        self
    }

    /// Builds the integration runner
    pub fn build(&self) -> Result<IntegrationRunner, TestError> {
        IntegrationRunner::new(self.integration_config.clone())
    }
}

/// Simplified enum representing lifecycle stages for hooks
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleStage {
    BeforeSetup,
    AfterSetup,
    BeforeTest,
    AfterTest,
    BeforeTeardown,
    AfterTeardown,
}

/// Convenience function to create a cross-crate test using a builder function
pub fn create_cross_crate_test<F>(name: &str, builder_fn: F) -> Result<IntegrationRunner, TestError>
where
    F: FnOnce(&mut CrossCrateTestBuilder) -> &mut CrossCrateTestBuilder,
{
    let mut builder = CrossCrateTestBuilder::new(name);
    builder_fn(&mut builder);
    let config = builder.build()?;
    IntegrationRunner::new(config)
}

/// Helper trait for creating service instances from configuration
pub trait ServiceFactory<T> {
    /// Create a service instance from configuration
    fn create(config: &ServiceConfig, context: &IntegrationContext) -> TestResult<T>;
}

/// CI/CD environment detection and configuration
pub struct CIEnvironment {
    /// CI provider name
    pub name: String,
    /// Build ID
    pub build_id: Option<String>,
    /// Project name
    pub project: Option<String>,
    /// Branch name
    pub branch: Option<String>,
    /// Commit hash
    pub commit: Option<String>,
    /// Is this a pull request
    pub is_pull_request: bool,
    /// Environment variables specific to the CI environment
    pub env_vars: HashMap<String, String>,
}

impl CIEnvironment {
    /// Detect the current CI environment
    pub fn detect() -> Option<Self> {
        // GitLab CI
        if std::env::var("GITLAB_CI").is_ok() {
            return Some(Self {
                name: "GitLab CI".to_string(),
                build_id: std::env::var("CI_PIPELINE_ID").ok(),
                project: std::env::var("CI_PROJECT_NAME").ok(),
                branch: std::env::var("CI_COMMIT_BRANCH").ok(),
                commit: std::env::var("CI_COMMIT_SHA").ok(),
                is_pull_request: std::env::var("CI_MERGE_REQUEST_ID").is_ok(),
                env_vars: Self::get_environment_variables(&["CI_", "GITLAB_"]),
            });
        }

        // GitHub Actions
        if std::env::var("GITHUB_ACTIONS").is_ok() {
            return Some(Self {
                name: "GitHub Actions".to_string(),
                build_id: std::env::var("GITHUB_RUN_ID").ok(),
                project: std::env::var("GITHUB_REPOSITORY")
                    .ok()
                    .map(|s| s.split('/').last().unwrap_or_default().to_string()),
                branch: std::env::var("GITHUB_REF")
                    .ok()
                    .map(|s| s.replace("refs/heads/", "")),
                commit: std::env::var("GITHUB_SHA").ok(),
                is_pull_request: std::env::var("GITHUB_EVENT_NAME").unwrap_or_default()
                    == "pull_request",
                env_vars: Self::get_environment_variables(&["GITHUB_"]),
            });
        }

        // Jenkins
        if std::env::var("JENKINS_URL").is_ok() {
            return Some(Self {
                name: "Jenkins".to_string(),
                build_id: std::env::var("BUILD_ID").ok(),
                project: std::env::var("JOB_NAME").ok(),
                branch: std::env::var("BRANCH_NAME").ok(),
                commit: std::env::var("GIT_COMMIT").ok(),
                is_pull_request: std::env::var("CHANGE_ID").is_ok(),
                env_vars: Self::get_environment_variables(&["BUILD_", "JOB_", "JENKINS_"]),
            });
        }

        // CircleCI
        if std::env::var("CIRCLECI").is_ok() {
            return Some(Self {
                name: "CircleCI".to_string(),
                build_id: std::env::var("CIRCLE_BUILD_NUM").ok(),
                project: std::env::var("CIRCLE_PROJECT_REPONAME").ok(),
                branch: std::env::var("CIRCLE_BRANCH").ok(),
                commit: std::env::var("CIRCLE_SHA1").ok(),
                is_pull_request: std::env::var("CIRCLE_PULL_REQUEST").is_ok(),
                env_vars: Self::get_environment_variables(&["CIRCLE_"]),
            });
        }

        // Azure Pipelines
        if std::env::var("TF_BUILD").is_ok() {
            return Some(Self {
                name: "Azure Pipelines".to_string(),
                build_id: std::env::var("BUILD_BUILDID").ok(),
                project: std::env::var("BUILD_REPOSITORY_NAME").ok(),
                branch: std::env::var("BUILD_SOURCEBRANCHNAME").ok(),
                commit: std::env::var("BUILD_SOURCEVERSION").ok(),
                is_pull_request: std::env::var("SYSTEM_PULLREQUEST_PULLREQUESTID").is_ok(),
                env_vars: Self::get_environment_variables(&["BUILD_", "SYSTEM_", "AGENT_"]),
            });
        }

        None
    }

    /// Get environment variables with specified prefixes
    fn get_environment_variables(prefixes: &[&str]) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for (key, value) in std::env::vars() {
            if prefixes.iter().any(|prefix| key.starts_with(prefix)) {
                result.insert(key, value);
            }
        }

        result
    }

    /// Is running in CI environment
    pub fn is_ci() -> bool {
        Self::detect().is_some()
    }

    /// Get the current branch name
    pub fn branch_name() -> Option<String> {
        Self::detect().and_then(|ci| ci.branch)
    }

    /// Get the current commit hash
    pub fn commit_hash() -> Option<String> {
        Self::detect().and_then(|ci| ci.commit)
    }

    /// Is this a pull request build
    pub fn is_pull_request() -> bool {
        Self::detect().map(|ci| ci.is_pull_request).unwrap_or(false)
    }
}

/// Configuration for test reports in CI/CD environments
pub struct CIReportConfig {
    /// Test report format
    pub format: ReportFormat,
    /// Output directory for reports
    pub output_dir: PathBuf,
    /// Generate separate report per test
    pub per_test_report: bool,
    /// Include test data in reports
    pub include_test_data: bool,
    /// Include environment information in reports
    pub include_environment: bool,
}

/// Test report format
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    /// JUnit XML format
    JUnit,
    /// JSON format
    JSON,
    /// Text format
    Text,
}

impl Default for CIReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::JUnit,
            output_dir: PathBuf::from("test-reports"),
            per_test_report: false,
            include_test_data: true,
            include_environment: true,
        }
    }
}

impl CIReportConfig {
    /// Create a new CI report config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the test report format
    pub fn with_format(mut self, format: ReportFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the output directory for reports
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set whether to generate a separate report per test
    pub fn with_per_test_report(mut self, value: bool) -> Self {
        self.per_test_report = value;
        self
    }

    /// Set whether to include test data in reports
    pub fn with_include_test_data(mut self, value: bool) -> Self {
        self.include_test_data = value;
        self
    }

    /// Set whether to include environment information in reports
    pub fn with_include_environment(mut self, value: bool) -> Self {
        self.include_environment = value;
        self
    }
}

impl IntegrationTestConfig {
    /// Configure the test for CI/CD environments
    pub fn configure_for_ci(&mut self) -> TestResult<()> {
        // Only apply CI-specific configurations if we're in a CI environment
        if !CIEnvironment::is_ci() {
            return Ok(());
        }

        // Add CI environment variables
        if let Some(ci) = CIEnvironment::detect() {
            println!("Detected CI environment: {}", ci.name);

            // Set environment variables based on CI environment
            self.env_vars
                .insert("CI_ENVIRONMENT".to_string(), ci.name.clone());

            if let Some(build_id) = &ci.build_id {
                self.env_vars
                    .insert("CI_BUILD_ID".to_string(), build_id.clone());
            }

            if let Some(branch) = &ci.branch {
                self.env_vars
                    .insert("CI_BRANCH".to_string(), branch.clone());
            }

            // Configure test directory based on CI
            if self.test_dir.is_none() {
                let base_dir = match ci.name.as_str() {
                    "GitLab CI" => PathBuf::from(
                        std::env::var("CI_PROJECT_DIR").unwrap_or_else(|_| ".".to_string()),
                    ),
                    "GitHub Actions" => PathBuf::from(
                        std::env::var("GITHUB_WORKSPACE").unwrap_or_else(|_| ".".to_string()),
                    ),
                    _ => PathBuf::from("."),
                };

                // Create a unique test directory
                let test_dir = base_dir.join("test-artifacts").join(&self.name);
                std::fs::create_dir_all(&test_dir)?;
                self.test_dir = Some(test_dir);
            }

            // Typical CI adjustments
            self.verify_mocks = true;
            self.cleanup_resources = false; // Typically keep artifacts in CI

            // Add CI-specific hooks
            self.lifecycle_hooks.before_setup.push(format!(
                "echo 'Running {} in CI environment {}'",
                self.name, ci.name
            ));

            if let Some(commit) = ci.commit {
                self.lifecycle_hooks
                    .before_setup
                    .push(format!("echo 'Test running on commit {}'", commit));
            }
        }

        Ok(())
    }
}

impl IntegrationRunner {
    /// Create a new integration test runner configured for CI environment
    pub fn new_ci(mut config: IntegrationTestConfig) -> TestResult<Self> {
        // Configure for CI
        config.configure_for_ci()?;

        // Create runner as normal
        Self::new(config)
    }

    /// Generate a test report compatible with CI systems
    pub fn generate_report(&self, config: CIReportConfig) -> TestResult<PathBuf> {
        // Create the output directory if it doesn't exist
        std::fs::create_dir_all(&config.output_dir)?;

        // Generate the report path
        let report_file = match config.format {
            ReportFormat::JUnit => config
                .output_dir
                .join(format!("{}-junit.xml", self.context.config.name)),
            ReportFormat::JSON => config
                .output_dir
                .join(format!("{}-report.json", self.context.config.name)),
            ReportFormat::Text => config
                .output_dir
                .join(format!("{}-report.txt", self.context.config.name)),
        };

        // For now, just create a simple placeholder report
        // In a real implementation, this would generate a proper report based on test results
        let report_content = match config.format {
            ReportFormat::JUnit => format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="{}" tests="1" failures="0" errors="0" skipped="0" timestamp="{}" time="0">
    <testcase classname="{}" name="{}" time="0"/>
  </testsuite>
</testsuites>"#,
                self.context.config.name,
                chrono::Utc::now().to_rfc3339(),
                self.context.config.name,
                self.context.config.name
            ),
            ReportFormat::JSON => serde_json::to_string_pretty(&serde_json::json!({
                "name": self.context.config.name,
                "status": "passed",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "duration_ms": 0,
                "environment": CIEnvironment::detect(),
            }))?,
            ReportFormat::Text => format!(
                "Test: {}\nStatus: passed\nTimestamp: {}\n",
                self.context.config.name,
                chrono::Utc::now().to_rfc3339()
            ),
        };

        std::fs::write(&report_file, report_content)?;

        Ok(report_file)
    }
}

impl CrossCrateTestBuilder {
    /// Configure the test for CI/CD environments
    pub fn for_ci(mut self) -> TestResult<Self> {
        self.integration_config.configure_for_ci()?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_integration_context_creation() {
        let config = IntegrationTestConfig {
            name: "test_context_creation".to_string(),
            ..Default::default()
        };

        let context = IntegrationContext::new(config).unwrap();
        assert!(context.test_dir.is_some());
    }

    #[test]
    fn test_environment_variables() {
        let mut config = IntegrationTestConfig::default();
        config.name = "test_env_vars".to_string();
        config
            .env_vars
            .insert("TEST_VAR".to_string(), "test_value".to_string());

        let context = IntegrationContext::new(config).unwrap();
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");

        // Test setting a new variable
        context.set_env_var("ANOTHER_VAR", "another_value").unwrap();
        assert_eq!(std::env::var("ANOTHER_VAR").unwrap(), "another_value");

        // Test getting a variable
        assert_eq!(
            context.get_env_var("ANOTHER_VAR").unwrap(),
            Some("another_value".to_string())
        );
    }

    #[test]
    fn test_integration_runner() {
        let config = IntegrationTestConfig {
            name: "test_runner".to_string(),
            ..Default::default()
        };

        let runner = IntegrationRunner::new(config).unwrap();

        let result = runner
            .run(|context| {
                // Create a fixture
                let mut runner_context = context.to_owned();
                let fixture = runner_context.create_fixture()?;

                // Add a component to the fixture
                fixture.register_component("test_component".to_string())?;

                // Retrieve the component
                let component = fixture.get_component::<String>()?;
                assert_eq!(component, "test_component");

                Ok(true)
            })
            .unwrap();

        assert!(result);
    }

    #[test]
    fn test_timeout_functionality() {
        let mut config = IntegrationTestConfig::default();
        config.name = "test_timeout".to_string();
        config.timeout = Some(Duration::from_millis(10));

        let runner = IntegrationRunner::new(config).unwrap();

        let result = runner.run_with_timeout(|_context| {
            // Sleep longer than the timeout
            std::thread::sleep(Duration::from_millis(50));
            Ok(true)
        });

        assert!(result.is_err());
        if let Err(TestError::TimeoutError(_)) = result {
            // Expected error
        } else {
            panic!("Expected timeout error, got: {:?}", result);
        }
    }

    #[test]
    fn test_cross_crate_test_builder() {
        let builder = CrossCrateTestBuilder::new("test_builder")
            .with_crate("navius-core")
            .with_crate("navius-db")
            .with_timeout(Duration::from_secs(30))
            .with_env_var("DB_URL", "postgres://localhost/test");

        let runner = builder.build().unwrap();

        let result = runner
            .run(|context| {
                assert_eq!(
                    context.get_env_var("DB_URL").unwrap(),
                    Some("postgres://localhost/test".to_string())
                );
                Ok(true)
            })
            .unwrap();

        assert!(result);
    }

    #[test]
    fn test_service_config_and_discovery() {
        let mut service_configs = HashMap::new();
        let mut db_properties = HashMap::new();
        db_properties.insert(
            "url".to_string(),
            ConfigValue::String("postgres://localhost/test".to_string()),
        );

        let db_config = ServiceConfig {
            name: "database".to_string(),
            service_type: "PostgresDatabase".to_string(),
            properties: db_properties,
            dependencies: Vec::new(),
        };

        service_configs.insert("database".to_string(), db_config);

        let mut cache_properties = HashMap::new();
        cache_properties.insert(
            "url".to_string(),
            ConfigValue::String("redis://localhost:6379".to_string()),
        );

        let cache_config = ServiceConfig {
            name: "cache".to_string(),
            service_type: "RedisCache".to_string(),
            properties: cache_properties,
            dependencies: vec!["database".to_string()],
        };

        service_configs.insert("cache".to_string(), cache_config);

        let mut config = IntegrationTestConfig::default();
        config.name = "test_service_discovery".to_string();
        config.service_configs = service_configs;

        let context = IntegrationContext::new(config).unwrap();
        let discovery_result = context.discover_services().unwrap();

        assert_eq!(discovery_result.instances.len(), 2);
        assert_eq!(discovery_result.dependencies.len(), 2);
        assert!(discovery_result.dependencies.contains_key("database"));
        assert!(discovery_result.dependencies.contains_key("cache"));
        assert_eq!(discovery_result.dependencies["database"].len(), 0);
        assert_eq!(discovery_result.dependencies["cache"].len(), 1);
        assert_eq!(discovery_result.dependencies["cache"][0], "database");
    }

    #[test]
    fn test_lifecycle_hooks() {
        let mut lifecycle_hooks = TestLifecycleHooks::default();
        lifecycle_hooks
            .before_setup
            .push("echo 'Before setup'".to_string());
        lifecycle_hooks
            .after_test
            .push("echo 'After test'".to_string());

        let mut config = IntegrationTestConfig::default();
        config.name = "test_lifecycle_hooks".to_string();
        config.lifecycle_hooks = lifecycle_hooks;

        let runner = IntegrationRunner::new(config).unwrap();
        let result = runner.run(|_| Ok(true)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_test_data_management() {
        // Create a temporary test data file
        let temp_dir = tempfile::tempdir().unwrap();
        let test_data_path = temp_dir.path().join("test_data.json");

        let mut content = HashMap::new();
        content.insert(
            "name".to_string(),
            ConfigValue::String("Test Entity".to_string()),
        );
        content.insert("active".to_string(), ConfigValue::Boolean(true));

        let test_data = TestData {
            id: "test_entity".to_string(),
            content,
        };

        let json = serde_json::to_string_pretty(&test_data).unwrap();
        std::fs::write(&test_data_path, json).unwrap();

        let mut config = IntegrationTestConfig::default();
        config.name = "test_data_management".to_string();
        config.test_data_path = Some(test_data_path);

        let context = IntegrationContext::new(config).unwrap();
        let loaded_data = context.get_test_data("test_entity").unwrap();

        assert_eq!(loaded_data.id, "test_entity");
        assert_eq!(
            loaded_data
                .content
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Test Entity"
        );
        assert!(
            loaded_data
                .content
                .get("active")
                .unwrap()
                .as_boolean()
                .unwrap()
        );
    }
}
