use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

use crate::error::{TestError, TestResult};

/// A registry for storing and retrieving mock objects by their interface type
#[derive(Debug, Default)]
pub struct MockRegistry {
    /// Internal storage for mock objects keyed by their interface type ID
    mocks: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,

    /// Expectations for method calls
    expectations: Arc<Mutex<Vec<Expectation>>>,
}

/// Represents an expected method call on a mock object
#[derive(Debug)]
pub struct Expectation {
    /// The name of the method
    pub method: String,

    /// The arguments to the method
    pub args: Vec<String>,

    /// The number of times the method is expected to be called
    pub times: ExpectedTimes,

    /// The number of times the method has been called
    pub called: usize,
}

/// Represents the number of times a method is expected to be called
#[derive(Debug, Clone, Copy)]
pub enum ExpectedTimes {
    /// The method is expected to be called exactly this many times
    Exact(usize),

    /// The method is expected to be called at least this many times
    AtLeast(usize),

    /// The method is expected to be called at most this many times
    AtMost(usize),

    /// The method is expected to be called any number of times
    Any,
}

impl fmt::Display for ExpectedTimes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectedTimes::Exact(n) => write!(f, "exactly {} times", n),
            ExpectedTimes::AtLeast(n) => write!(f, "at least {} times", n),
            ExpectedTimes::AtMost(n) => write!(f, "at most {} times", n),
            ExpectedTimes::Any => write!(f, "any number of times"),
        }
    }
}

impl Expectation {
    /// Create a new expectation
    pub fn new<S: Into<String>>(method: S) -> Self {
        Self {
            method: method.into(),
            args: Vec::new(),
            times: ExpectedTimes::Any,
            called: 0,
        }
    }

    /// Set the arguments for the expectation
    pub fn with_args<S: Into<String>>(mut self, args: Vec<S>) -> Self {
        self.args = args.into_iter().map(|a| a.into()).collect();
        self
    }

    /// Set the number of times the method is expected to be called
    pub fn times(mut self, times: ExpectedTimes) -> Self {
        self.times = times;
        self
    }

    /// Record a call to the method
    pub fn record_call(&mut self) {
        self.called += 1;
    }

    /// Verify that the expectation has been met
    pub fn verify(&self) -> TestResult<()> {
        match self.times {
            ExpectedTimes::Exact(n) if self.called != n => {
                Err(TestError::ExpectationNotMet(format!(
                    "Method '{}' was expected to be called {} times, but was called {} times",
                    self.method, n, self.called
                )))
            }
            ExpectedTimes::AtLeast(n) if self.called < n => {
                Err(TestError::ExpectationNotMet(format!(
                    "Method '{}' was expected to be called at least {} times, but was called {} times",
                    self.method, n, self.called
                )))
            }
            ExpectedTimes::AtMost(n) if self.called > n => {
                Err(TestError::ExpectationNotMet(format!(
                    "Method '{}' was expected to be called at most {} times, but was called {} times",
                    self.method, n, self.called
                )))
            }
            _ => Ok(()),
        }
    }
}

impl MockRegistry {
    /// Create a new mock registry
    pub fn new() -> Self {
        Self {
            mocks: RwLock::new(HashMap::new()),
            expectations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a mock implementation for a given interface
    pub fn register<I: ?Sized + 'static, M: Any + Send + Sync>(
        &self,
        mock: Arc<M>,
    ) -> TestResult<()> {
        let type_id = TypeId::of::<I>();
        let mut mocks = self.mocks.write().map_err(|_| {
            TestError::RegistryError("Failed to acquire write lock on mocks".to_string())
        })?;

        let mock_any: Box<dyn Any + Send + Sync> = Box::new(mock);
        mocks.insert(type_id, mock_any);

        Ok(())
    }

    /// Get a mock implementation for a given interface
    pub fn get<I: ?Sized + 'static, M: 'static>(&self) -> TestResult<Arc<M>> {
        let type_id = TypeId::of::<I>();
        let mocks = self.mocks.read().map_err(|_| {
            TestError::RegistryError("Failed to acquire read lock on mocks".to_string())
        })?;

        let mock_any = mocks.get(&type_id).ok_or_else(|| {
            TestError::MockNotFound(format!("Mock for interface {:?} not found", type_id))
        })?;

        let mock_arc = mock_any.downcast_ref::<Arc<M>>().ok_or_else(|| {
            TestError::MockTypeMismatch(format!(
                "Mock for interface {:?} is not of the expected type",
                type_id
            ))
        })?;

        Ok(Arc::clone(mock_arc))
    }

    /// Add an expectation to the registry
    pub fn expect(&self, expectation: Expectation) -> TestResult<()> {
        let mut expectations = self.expectations.lock().map_err(|_| {
            TestError::RegistryError("Failed to acquire lock on expectations".to_string())
        })?;

        expectations.push(expectation);

        Ok(())
    }

    /// Record a method call
    pub fn record_call<S: Into<String>>(&self, method: S, args: &[String]) -> TestResult<()> {
        let mut expectations = self.expectations.lock().map_err(|_| {
            TestError::RegistryError("Failed to acquire lock on expectations".to_string())
        })?;

        let method = method.into();
        for expectation in expectations.iter_mut() {
            if expectation.method == method {
                // TODO: Match arguments
                expectation.record_call();
                break;
            }
        }

        Ok(())
    }

    /// Verify that all expectations have been met
    pub fn verify(&self) -> TestResult<()> {
        let expectations = self.expectations.lock().map_err(|_| {
            TestError::RegistryError("Failed to acquire lock on expectations".to_string())
        })?;

        for expectation in expectations.iter() {
            expectation.verify()?;
        }

        Ok(())
    }

    /// Reset all expectations
    pub fn reset(&self) -> TestResult<()> {
        let mut expectations = self.expectations.lock().map_err(|_| {
            TestError::RegistryError("Failed to acquire lock on expectations".to_string())
        })?;

        expectations.clear();

        Ok(())
    }

    /// Clear all registered mocks
    pub fn clear(&self) -> TestResult<()> {
        let mut mocks = self.mocks.write().map_err(|_| {
            TestError::RegistryError("Failed to acquire write lock on mocks".to_string())
        })?;

        mocks.clear();

        Ok(())
    }
}

/// Module for mock configuration utilities
pub mod config;

/// Module for mock registry events
pub mod events;

/// Module for mock expectation APIs
pub mod expect;

// Re-export common types
pub use config::MockConfig;
pub use events::MockEvent;
pub use expect::MockExpect;

#[cfg(test)]
mod tests {
    use super::*;

    // Define a test interface
    trait TestInterface: Send + Sync {
        fn test_method(&self, arg: &str) -> String;
    }

    // Define a mock implementation
    struct MockTestInterface {
        calls: Mutex<Vec<String>>,
    }

    impl MockTestInterface {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl TestInterface for MockTestInterface {
        fn test_method(&self, arg: &str) -> String {
            let mut calls = self.calls.lock().unwrap();
            calls.push(arg.to_string());
            format!("Mocked: {}", arg)
        }
    }

    #[test]
    fn test_mock_registry() {
        // Create a new registry
        let registry = MockRegistry::new();

        // Register a mock
        let mock = Arc::new(MockTestInterface::new());
        registry
            .register::<dyn TestInterface, _>(Arc::clone(&mock))
            .unwrap();

        // Get the mock back
        let retrieved: Arc<MockTestInterface> = registry.get::<dyn TestInterface, _>().unwrap();

        // Test that it's the same mock
        let result = retrieved.test_method("test");
        assert_eq!(result, "Mocked: test");
    }

    #[test]
    fn test_expectations() {
        let registry = MockRegistry::new();

        // Add an expectation
        let expectation = Expectation::new("test_method")
            .with_args(vec!["test"])
            .times(ExpectedTimes::Exact(1));

        registry.expect(expectation).unwrap();

        // Record a call
        registry
            .record_call("test_method", &["test".to_string()])
            .unwrap();

        // Verify expectations
        registry.verify().unwrap();
    }

    #[test]
    fn test_expectation_not_met() {
        let registry = MockRegistry::new();

        // Add an expectation
        let expectation = Expectation::new("test_method")
            .with_args(vec!["test"])
            .times(ExpectedTimes::Exact(2));

        registry.expect(expectation).unwrap();

        // Record a call
        registry
            .record_call("test_method", &["test".to_string()])
            .unwrap();

        // Verify expectations - should fail
        let result = registry.verify();
        assert!(result.is_err());
    }
}
