// Export base types for mocking
pub mod config;
pub mod events;
pub mod expect;

// Define the Expectation type and other key types
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

pub use self::config::{Expectation, ExpectedTimes};
pub use self::expect::{MethodExpectBuilder, MockExpectBuilder};
use crate::error::{TestError, TestResult};

// Core traits and structures for mocking
pub trait MockProvider {
    fn register(&self, registry: &MockRegistry) -> Arc<Self>;
}

pub trait MockVerify {
    fn verify(&self) -> TestResult<()>;
}

#[derive(Debug)]
pub struct MockRegistry {
    mocks: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    expectations: RwLock<HashMap<String, Vec<Expectation>>>,
    call_counts: RwLock<HashMap<String, usize>>,
}

impl MockRegistry {
    pub fn new() -> Self {
        MockRegistry {
            mocks: RwLock::new(HashMap::new()),
            expectations: RwLock::new(HashMap::new()),
            call_counts: RwLock::new(HashMap::new()),
        }
    }

    /// Register a mock component
    pub fn register<T: ?Sized + 'static, M: 'static + Send + Sync>(
        &self,
        mock: Arc<M>,
    ) -> TestResult<()> {
        let type_id = TypeId::of::<T>();
        let boxed_mock = Box::new(mock);
        let dyn_mock = boxed_mock as Box<dyn Any + Send + Sync>;

        let mut mocks = self.mocks.write().unwrap();
        mocks.insert(type_id, dyn_mock);

        Ok(())
    }

    /// Register a mock component without the additional parameters
    pub fn register_mock<T: ?Sized + 'static>(&self, mock: Arc<impl Any + Send + Sync + 'static>) {
        let type_id = TypeId::of::<T>();
        let boxed_mock = Box::new(mock);
        let dyn_mock = boxed_mock as Box<dyn Any + Send + Sync>;

        let mut mocks = self.mocks.write().unwrap();
        mocks.insert(type_id, dyn_mock);
    }

    /// Get a mock component
    pub fn get<T: ?Sized + 'static>(&self) -> TestResult<Arc<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<T>();
        let mocks = self.mocks.read().unwrap();

        if let Some(mock) = mocks.get(&type_id) {
            // SAFETY: We're downcasting to the type we registered with
            let arc_any = mock.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
            Ok(arc_any.clone())
        } else {
            Err(TestError::missing_component(format!(
                "Mock for {:?} not found",
                type_id
            )))
        }
    }

    /// Set up an expectation for a specific mock and method
    pub fn expect<T: 'static>(
        &self,
        mock_name: impl Into<String>,
        method: impl Into<String>,
    ) -> MethodExpectBuilder<'_, T> {
        let mock_name = mock_name.into();
        let method = method.into();
        MethodExpectBuilder::new(self, mock_name, method)
    }

    /// Create a builder for setting up expectations on a mock
    pub fn expect_on<T: 'static>(&self, mock_name: impl Into<String>) -> MockExpectBuilder<'_, T> {
        MockExpectBuilder::new(self, mock_name)
    }

    /// Add an expectation to the registry
    pub fn add_expectation(
        &self,
        mock_name: impl Into<String>,
        expectation: Expectation,
    ) -> TestResult<()> {
        let key = mock_name.into();
        let mut expectations = self.expectations.write().unwrap();

        let mock_expectations = expectations.entry(key).or_insert_with(Vec::new);
        mock_expectations.push(expectation);

        Ok(())
    }

    /// Record a method call
    pub fn record_call(
        &self,
        mock_name: impl Into<String> + Clone,
        method: impl Into<String> + Clone,
        args: Vec<String>,
    ) -> TestResult<()> {
        let mock_name_clone = mock_name.clone();
        let key = format!("{}::{}", mock_name.into(), method.clone().into());
        let mut call_counts = self.call_counts.write().unwrap();

        let count = call_counts.entry(key.clone()).or_insert(0);
        *count += 1;

        // Check if this call matches any expectations
        let expectations = self.expectations.read().unwrap();
        if let Some(mock_expectations) = expectations.get(&mock_name_clone.into()) {
            for expectation in mock_expectations {
                if expectation.method == method.clone().into() && expectation.args == args {
                    // Found a matching expectation
                    return Ok(());
                }
            }
        }

        // No matching expectation found, but we'll still allow the call
        Ok(())
    }

    /// Reset the registry
    pub fn reset(&self) -> TestResult<()> {
        let mut expectations = self.expectations.write().unwrap();
        let mut call_counts = self.call_counts.write().unwrap();

        expectations.clear();
        call_counts.clear();

        Ok(())
    }

    /// Verify all mocks
    pub fn verify(&self) -> TestResult<()> {
        let mocks = self.mocks.read().unwrap();
        let expectations = self.expectations.read().unwrap();
        let call_counts = self.call_counts.read().unwrap();

        let mut errors = Vec::new();

        // Check each expectation
        for (mock_name, mock_expectations) in expectations.iter() {
            for expectation in mock_expectations {
                let call_key = format!("{}::{}", mock_name, expectation.method);
                let actual_calls = call_counts.get(&call_key).copied().unwrap_or(0);

                if !expectation.times.is_satisfied(actual_calls) {
                    errors.push(format!(
                        "Expectation not met for {}::{}: expected {:?}, got {}",
                        mock_name, expectation.method, expectation.times, actual_calls
                    ));
                }
            }
        }

        // Also verify any registered mockable objects
        for (_, mock) in mocks.iter() {
            if let Some(verifiable) = mock.downcast_ref::<Arc<dyn MockVerify + Send + Sync>>() {
                if let Err(e) = verifiable.verify() {
                    errors.push(format!("Mock verification failed: {}", e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TestError::mock_expectation_error(format!(
                "Mock verification failed:\n{}",
                errors.join("\n")
            )))
        }
    }

    pub fn verify_call<S1, S2>(&self, mock_name: S1, method: S2, args: Vec<String>) -> bool
    where
        S1: Into<String> + Clone,
        S2: Into<String> + Clone,
    {
        let mock_name_str = mock_name.clone().into();
        let method_str = method.clone().into();
        let key = format!("{}::{}", mock_name_str, method_str);

        if let Some(mock_expectations) = self.expectations.read().unwrap().get(&mock_name_str) {
            for expectation in mock_expectations {
                if expectation.method == method_str && expectation.args == args {
                    return true;
                }
            }
        }
        false
    }

    pub fn verify_expectation(
        &self,
        mock_name: impl AsRef<str>,
        method: impl AsRef<str>,
        args: Vec<String>,
    ) -> bool {
        let mock_name = mock_name.as_ref();
        let method = method.as_ref();
        let key = format!("{}::{}", mock_name, method);

        if let Some(mock_expectations) = self.expectations.read().unwrap().get(mock_name) {
            for expectation in mock_expectations {
                if expectation.method == method && expectation.args == args {
                    return true;
                }
            }
        }
        false
    }
}

// Re-export from the config, events, and expect modules
pub use self::config::*;
pub use self::events::*;
pub use self::expect::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_registry() -> TestResult<()> {
        let registry = MockRegistry::new();

        // Test adding an expectation
        let expectation = Expectation::new("test_method")
            .with_args(vec!["arg1", "arg2"])
            .times(ExpectedTimes::Exact(1));

        registry.add_expectation("TestMock", expectation)?;

        // Test recording a call
        registry.record_call(
            "TestMock",
            "test_method",
            vec!["arg1".to_string(), "arg2".to_string()],
        )?;

        // Verification should pass
        registry.verify()?;

        Ok(())
    }

    #[test]
    fn test_mock_registry_failure() {
        let registry = MockRegistry::new();

        // Add an expectation that won't be satisfied
        let expectation = Expectation::new("test_method")
            .with_args(vec!["arg1", "arg2"])
            .times(ExpectedTimes::Exact(1));

        registry.add_expectation("TestMock", expectation).unwrap();

        // Don't record any calls

        // Verification should fail
        let result = registry.verify();
        assert!(result.is_err());
    }
}
