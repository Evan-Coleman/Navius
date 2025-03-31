//! # Navius Test Framework
//!
//! A comprehensive testing framework for the Navius application, focusing on
//! cross-crate testing, error injection, and mock implementations.
//!
//! ## Main Features
//!
//! - **Mock Registry**: A system for registering and retrieving mock implementations
//!   of interfaces used throughout the Navius application.
//!
//! - **Test Fixtures**: A system for setting up test environments with specific
//!   components and resources.
//!
//! - **Error Injection**: Utilities for testing error handling and propagation
//!   across component boundaries.
//!
//! - **Assertion Utilities**: Helper functions for verifying test conditions
//!   and mock expectations.
//!
//! - **Integration Test Utilities**: Tools for building and running cross-crate
//!   integration tests.
//!
//! ## Example
//!
//! ```rust
//! use navius_test::{
//!     fixture::TestFixture,
//!     mock::MockRegistry,
//!     mocks::database::MockDatabaseClient,
//!     error::TestResult,
//! };
//!
//! fn test_with_mock_db() -> TestResult<()> {
//!     // Create a test fixture and mock registry
//!     let fixture = TestFixture::new();
//!     let registry = MockRegistry::new();
//!
//!     // Register the registry with the fixture
//!     fixture.register_component(registry.clone())?;
//!
//!     // Create and register a mock database
//!     let mock_db = MockDatabaseClient::new(registry.clone());
//!     
//!     // Configure expectations on the mock
//!     registry.expect("DatabaseClient", "query")
//!         .with(("SELECT * FROM users", vec![]))
//!         .times(1)
//!         .returns(Ok(vec![
//!             // Row data here
//!         ]));
//!
//!     // Use the mock in a test
//!     let result = mock_db.query("SELECT * FROM users", &[])?;
//!     
//!     // Verify the mock expectations
//!     registry.verify()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Integration Test Example
//!
//! ```rust
//! use navius_test::{
//!     integration::{CrossCrateTestBuilder, IntegrationTestConfig},
//!     config::TestConfigBuilder,
//!     test_case, test_suite,
//!     error::TestResult,
//! };
//!
//! fn run_integration_tests() -> TestResult<()> {
//!     // Create test configuration
//!     let config = TestConfigBuilder::new("integration_tests")
//!         .with_description("Core API integration tests")
//!         .with_test_timeout(60)
//!         .build();
//!
//!     // Create test cases
//!     let test1 = test_case!("user_service_test", |context| {
//!         // Test the user service with mock dependencies
//!         let registry = context.registry();
//!         
//!         // Set up expectations
//!         registry.expect("DatabaseClient", "query")
//!             .with(("SELECT * FROM users WHERE id = ?", vec![1]))
//!             .returns(Ok(vec![
//!                 // User data
//!             ]));
//!         
//!         // Run the test
//!         let user_service = UserService::new(context.get_component()?);
//!         let user = user_service.get_user(1)?;
//!         
//!         // Verify the result
//!         assert_eq!(user.id, 1);
//!         assert_eq!(user.name, "Test User");
//!         
//!         Ok(())
//!     });
//!     
//!     // Create and run the test suite
//!     let runner = test_suite!("api_tests", config, test1);
//!     let reports = runner.run_all()?;
//!     
//!     // Check for failures
//!     let failures = reports.iter().filter(|r| !r.passed).count();
//!     assert_eq!(failures, 0, "Some tests failed");
//!     
//!     Ok(())
//! }
//! ```

// Re-export commonly used types and modules
pub mod config;
pub mod error;
pub mod fixture;
pub mod harness;
pub mod integration;
pub mod mock;
pub mod mocks;
pub mod runner;

// Re-export common types for easier usage
pub use error::{TestError, TestResult};
pub use fixture::TestFixture;
pub use harness::TestHarness;
pub use mock::MockRegistry;

// Re-export common mocks types for convenience
pub use mocks::{
    CommonMocks, HasMockAuth, HasMockConfiguration, HasMockDatabase, HasMockEventBroker,
    HasMockFileSystem, HasMockHttpClient, HasMockLogger, HasMockMessageBroker, HasMockMetrics,
    HasMockRbac, MockFixture, setup_common_mocks,
};

/// Error testing module for testing error handling
///
/// This module provides utilities for injecting errors during testing,
/// tracking error propagation through components, and verifying that errors
/// are properly handled.
pub mod error_testing {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};

    use crate::error::TestResult;

    /// Error that can be injected during testing
    #[derive(Debug, Clone)]
    pub struct InjectedError {
        /// Name of the error point
        pub name: String,
        /// Error message
        pub message: String,
        /// Context for the error
        pub context: HashMap<String, String>,
    }

    impl InjectedError {
        /// Create a new injected error with the given name and message
        pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                message: message.into(),
                context: HashMap::new(),
            }
        }

        /// Add context to the error
        pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.context.insert(key.into(), value.into());
            self
        }
    }

    /// Registry for error injection points
    #[derive(Debug, Default)]
    pub struct ErrorRegistry {
        /// Enabled error injection points
        enabled: RwLock<HashMap<String, InjectedError>>,
    }

    impl ErrorRegistry {
        /// Create a new error registry
        pub fn new() -> Self {
            Self {
                enabled: RwLock::new(HashMap::new()),
            }
        }

        /// Enable an error injection point
        pub fn enable(&self, error: InjectedError) -> TestResult<()> {
            let mut enabled = self.enabled.write().unwrap();
            enabled.insert(error.name.clone(), error);
            Ok(())
        }

        /// Disable an error injection point
        pub fn disable(&self, name: &str) -> TestResult<()> {
            let mut enabled = self.enabled.write().unwrap();
            enabled.remove(name);
            Ok(())
        }

        /// Check if an error should be injected at the given point
        pub fn should_inject(&self, name: &str) -> Option<InjectedError> {
            let enabled = self.enabled.read().unwrap();
            enabled.get(name).cloned()
        }

        /// Clear all error injection points
        pub fn clear(&self) -> TestResult<()> {
            let mut enabled = self.enabled.write().unwrap();
            enabled.clear();
            Ok(())
        }
    }

    /// Tracker for error propagation through components
    #[derive(Debug, Default)]
    pub struct ErrorTracker {
        /// Tracked errors
        errors: Mutex<Vec<TrackedError>>,
    }

    /// Error tracked during testing
    #[derive(Debug, Clone)]
    pub struct TrackedError {
        /// Name of the error point
        pub name: String,
        /// Error message
        pub message: String,
        /// Context for the error
        pub context: HashMap<String, String>,
        /// Component path where the error was tracked
        pub path: Vec<String>,
    }

    impl ErrorTracker {
        /// Create a new error tracker
        pub fn new() -> Self {
            Self {
                errors: Mutex::new(Vec::new()),
            }
        }

        /// Track an error
        pub fn track(
            &self,
            name: impl Into<String>,
            message: impl Into<String>,
            path: Vec<String>,
        ) -> TestResult<()> {
            let mut errors = self.errors.lock().unwrap();
            errors.push(TrackedError {
                name: name.into(),
                message: message.into(),
                context: HashMap::new(),
                path,
            });
            Ok(())
        }

        /// Track an error with context
        pub fn track_with_context(
            &self,
            name: impl Into<String>,
            message: impl Into<String>,
            path: Vec<String>,
            context: HashMap<String, String>,
        ) -> TestResult<()> {
            let mut errors = self.errors.lock().unwrap();
            errors.push(TrackedError {
                name: name.into(),
                message: message.into(),
                context,
                path,
            });
            Ok(())
        }

        /// Get all tracked errors
        pub fn errors(&self) -> Vec<TrackedError> {
            let errors = self.errors.lock().unwrap();
            errors.clone()
        }

        /// Clear all tracked errors
        pub fn clear(&self) -> TestResult<()> {
            let mut errors = self.errors.lock().unwrap();
            errors.clear();
            Ok(())
        }

        /// Verify that a specific error was tracked
        pub fn verify_error(&self, name: &str) -> TestResult<Vec<TrackedError>> {
            let errors = self.errors.lock().unwrap();
            let matching = errors
                .iter()
                .filter(|e| e.name == name)
                .cloned()
                .collect::<Vec<_>>();

            if matching.is_empty() {
                return Err(crate::error::TestError::AssertionFailed(format!(
                    "No error with name '{}' was tracked",
                    name
                )));
            }

            Ok(matching)
        }

        /// Verify that a specific error was tracked with the given context
        pub fn verify_error_with_context(
            &self,
            name: &str,
            context_key: &str,
            context_value: &str,
        ) -> TestResult<Vec<TrackedError>> {
            let errors = self.errors.lock().unwrap();
            let matching = errors
                .iter()
                .filter(|e| {
                    e.name == name
                        && e.context
                            .get(context_key)
                            .map(|v| v == context_value)
                            .unwrap_or(false)
                })
                .cloned()
                .collect::<Vec<_>>();

            if matching.is_empty() {
                return Err(crate::error::TestError::AssertionFailed(format!(
                    "No error with name '{}' and context '{}: {}' was tracked",
                    name, context_key, context_value
                )));
            }

            Ok(matching)
        }

        /// Verify that an error was propagated through the given component path
        pub fn verify_propagation(
            &self,
            name: &str,
            path: &[&str],
        ) -> TestResult<Vec<TrackedError>> {
            let errors = self.errors.lock().unwrap();
            let matching = errors
                .iter()
                .filter(|e| e.name == name && path.iter().all(|p| e.path.iter().any(|ep| ep == p)))
                .cloned()
                .collect::<Vec<_>>();

            if matching.is_empty() {
                return Err(crate::error::TestError::AssertionFailed(format!(
                    "No error with name '{}' was propagated through path {:?}",
                    name, path
                )));
            }

            Ok(matching)
        }
    }

    /// Combined error testing utilities
    #[derive(Debug, Default)]
    pub struct ErrorTesting {
        /// Error registry for injection
        pub registry: Arc<ErrorRegistry>,
        /// Error tracker for propagation
        pub tracker: Arc<ErrorTracker>,
    }

    impl ErrorTesting {
        /// Create a new error testing utility
        pub fn new() -> Self {
            Self {
                registry: Arc::new(ErrorRegistry::new()),
                tracker: Arc::new(ErrorTracker::new()),
            }
        }

        /// Reset all error testing state
        pub fn reset(&self) -> TestResult<()> {
            self.registry.clear()?;
            self.tracker.clear()?;
            Ok(())
        }
    }

    /// Macro for injecting errors at specific points
    #[macro_export]
    macro_rules! inject_error {
        ($registry:expr, $name:expr, $error_type:ty, $msg:expr) => {
            if let Some(injected) = $registry.should_inject($name) {
                return Err(<$error_type>::from(injected.message.clone()));
            }
        };
    }

    /// Macro for tracking error propagation
    #[macro_export]
    macro_rules! track_error {
        ($tracker:expr, $name:expr, $path:expr, $err:expr) => {
            let _ = $tracker.track($name, $err.to_string(), $path.to_vec());
        };
    }

    /// Macro for verifying error handling
    #[macro_export]
    macro_rules! verify_error {
        ($tracker:expr, $name:expr) => {
            $tracker.verify_error($name)?;
        };

        ($tracker:expr, $name:expr, $context_key:expr, $context_value:expr) => {
            $tracker.verify_error_with_context($name, $context_key, $context_value)?;
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TestResult;
    use crate::error_testing::{ErrorRegistry, ErrorTracker, InjectedError};
    use crate::fixture::TestFixture;
    use crate::harness::TestHarness;
    use crate::mock::MockRegistry;

    #[test]
    fn test_error_injection() -> TestResult<()> {
        let registry = ErrorRegistry::new();

        // Enable an error injection point
        registry.enable(InjectedError::new("test_error", "Injected error"))?;

        // Check if the error should be injected
        let injected = registry.should_inject("test_error");
        assert!(injected.is_some());
        assert_eq!(injected.unwrap().message, "Injected error");

        // Disable the error injection point
        registry.disable("test_error")?;

        // Check that the error is no longer injected
        let injected = registry.should_inject("test_error");
        assert!(injected.is_none());

        Ok(())
    }

    #[test]
    fn test_error_tracking() -> TestResult<()> {
        let tracker = ErrorTracker::new();

        // Track an error
        tracker.track(
            "test_error",
            "Test error message",
            vec!["component1".into(), "component2".into()],
        )?;

        // Verify the error was tracked
        let errors = tracker.verify_error("test_error")?;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error message");
        assert_eq!(errors[0].path, vec!["component1", "component2"]);

        // Track an error with context
        let mut context = std::collections::HashMap::new();
        context.insert("user_id".into(), "123".into());

        tracker.track_with_context(
            "context_error",
            "Error with context",
            vec!["component3".into()],
            context,
        )?;

        // Verify the error with context
        let errors = tracker.verify_error_with_context("context_error", "user_id", "123")?;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error with context");
        assert_eq!(errors[0].path, vec!["component3"]);

        // Verify error propagation
        let errors = tracker.verify_propagation("test_error", &["component1", "component2"])?;
        assert_eq!(errors.len(), 1);

        // Clear the tracker
        tracker.clear()?;

        // Verify there are no more errors
        let errors = tracker.errors();
        assert!(errors.is_empty());

        Ok(())
    }

    #[test]
    fn test_complete_test_workflow() -> TestResult<()> {
        // Create test fixture and registry
        let fixture = TestFixture::new();
        let registry = MockRegistry::new();

        // Register the registry with the fixture
        fixture.register_component(registry.clone())?;

        // Set up error testing
        let error_registry = ErrorRegistry::new();
        let error_tracker = ErrorTracker::new();

        fixture.register_component(Arc::new(error_registry))?;
        fixture.register_component(Arc::new(error_tracker))?;

        // Create a test harness
        let harness = TestHarness::new().with_fixture(fixture);

        // Run a test with the harness
        harness.run(|fixture| {
            let registry = fixture.get_component::<MockRegistry>()?;
            let _error_registry = fixture.get_component::<Arc<ErrorRegistry>>()?;
            let _error_tracker = fixture.get_component::<Arc<ErrorTracker>>()?;

            // Set up some expectations
            registry
                .expect("TestComponent", "test_method")
                .times(1)
                .returns(42);

            // Verify the expectations
            registry.verify()?;

            Ok(())
        })?;

        Ok(())
    }
}
