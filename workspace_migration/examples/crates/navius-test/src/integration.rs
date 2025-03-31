//! Integration Test Utilities
//!
//! This module provides utilities for creating and running integration tests
//! that span multiple crates in the Navius workspace. It builds upon the
//! test fixture, mock registry, and test harness components to provide
//! a comprehensive testing environment.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::error::{TestError, TestResult};
use crate::fixture::TestFixture;
use crate::harness::{TestHarness, TestOptions};
use crate::mock::MockRegistry;

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
        }
    }
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
pub struct IntegrationContext {
    config: IntegrationTestConfig,
    fixtures: Vec<Arc<TestFixture>>,
    test_dir: Option<PathBuf>,
    env_vars: RwLock<HashMap<String, String>>,
    registry: Arc<MockRegistry>,
    original_env: HashMap<String, Option<String>>,
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

        Ok(Self {
            config,
            fixtures: Vec::new(),
            test_dir,
            env_vars: RwLock::new(HashMap::new()),
            registry,
            original_env,
        })
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
        let mut env_vars = self.env_vars.write().map_err(|_| {
            TestError::ConcurrencyError("Failed to acquire write lock for env vars".into())
        })?;
        env_vars.insert(key.to_string(), value.to_string());
        std::env::set_var(key, value);
        Ok(())
    }

    /// Get an environment variable set for the test
    pub fn get_env_var(&self, key: &str) -> TestResult<Option<String>> {
        let env_vars = self.env_vars.read().map_err(|_| {
            TestError::ConcurrencyError("Failed to acquire read lock for env vars".into())
        })?;
        Ok(env_vars.get(key).cloned())
    }

    /// Get the mock registry
    pub fn registry(&self) -> Arc<MockRegistry> {
        self.registry.clone()
    }

    /// Create a test harness with this context's registry
    pub fn create_harness(&self) -> TestHarness {
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
            .with_fixture(fixture)
            .with_options(options)
    }
}

impl Drop for IntegrationContext {
    fn drop(&mut self) {
        // Restore original environment variables
        for (key, value) in &self.original_env {
            match value {
                Some(val) => unsafe { std::env::set_var(key, val) },
                None => unsafe { std::env::remove_var(key) },
            }
        }

        // Clean up test directory if cleanup_resources is true
        if self.config.cleanup_resources {
            if let Some(dir) = &self.test_dir {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
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
        let context = IntegrationContext::new(config)?;
        Ok(Self { context })
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
        // Set up environment variables
        for (key, value) in self.context.config.env_vars.iter() {
            std::env::set_var(key, value);
        }

        // Run the test
        let result = test_fn(&self.context);

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
            let context = &self.context;

            let handle = thread::spawn(move || {
                let result = test_fn(context);
                let _ = tx.send(result);
            });

            let start = Instant::now();
            let result = rx.recv_timeout(timeout);

            match result {
                Ok(test_result) => test_result,
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
}

impl Default for CrossCrateTestConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_cross_crate_test".to_string(),
            crates: Vec::new(),
            timeout: Some(Duration::from_secs(60)),
            resources_dir: None,
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
    /// Create a new cross-crate test builder
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let integration_config = IntegrationTestConfig {
            name: name.clone(),
            ..Default::default()
        };
        Self {
            config: CrossCrateTestConfig {
                name,
                ..Default::default()
            },
            integration_config,
            fixtures: HashMap::new(),
        }
    }

    /// Add a crate to the test
    pub fn with_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.config.crates.push(crate_name.into());
        self
    }

    /// Set the test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self.integration_config.timeout = Some(timeout);
        self
    }

    /// Set the resources directory
    pub fn with_resources_dir(mut self, dir: PathBuf) -> Self {
        self.config.resources_dir = Some(dir.clone());
        self.integration_config.test_dir = Some(dir);
        self
    }

    /// Set an environment variable for the test
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.integration_config
            .env_vars
            .insert(key.into(), value.into());
        self
    }

    /// Create a test runner from this builder
    pub fn build(self) -> TestResult<IntegrationRunner> {
        IntegrationRunner::new(self.integration_config)
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
}
