use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use super::{Expectation, ExpectedTimes, MockRegistry};
use crate::error::{TestError, TestResult};

/// Configuration for a mock object
#[derive(Debug)]
pub struct MockConfig<T> {
    /// Registry for the mock
    registry: Arc<MockRegistry>,

    /// Type marker
    _marker: PhantomData<T>,
}

/// An expected method call with specific arguments
#[derive(Debug)]
pub struct MethodExpectation<T> {
    /// The name of the method
    method: String,

    /// The registry that holds the expectation
    registry: Arc<MockRegistry>,

    /// The expectation being configured
    expectation: Expectation,

    /// Type marker
    _marker: PhantomData<T>,
}

/// Builder for setting up mock expectations
pub struct MockExpectBuilder<T> {
    /// Method name
    method: String,

    /// Registry for the expectation
    registry: Arc<MockRegistry>,

    /// Type marker
    _marker: PhantomData<T>,
}

/// Expected return value for a method
#[derive(Debug)]
pub enum ReturnValue<T> {
    /// The method will return a specific value
    Value(Box<dyn Any + Send + Sync>),

    /// The method will return a value based on the arguments
    Function(Box<dyn Fn(&[String]) -> Box<dyn Any + Send + Sync> + Send + Sync>),

    /// The method will return different values on successive calls
    Sequence(Vec<Box<dyn Any + Send + Sync>>),

    /// Type marker
    _Phantom(PhantomData<T>),
}

impl<T> MockConfig<T> {
    /// Create a new mock configuration
    pub fn new(registry: &Arc<MockRegistry>) -> Self {
        Self {
            registry: Arc::clone(registry),
            _marker: PhantomData,
        }
    }

    /// Expect a method call
    pub fn expect<S: Into<String>>(&self, method: S) -> MockExpectBuilder<T> {
        MockExpectBuilder {
            method: method.into(),
            registry: Arc::clone(&self.registry),
            _marker: PhantomData,
        }
    }

    /// Verify that all expectations have been met
    pub fn verify(&self) -> TestResult<()> {
        self.registry.verify()
    }

    /// Reset all expectations
    pub fn reset(&self) -> TestResult<()> {
        self.registry.reset()
    }
}

impl<T> MockExpectBuilder<T> {
    /// Set the arguments for the expectation
    pub fn with_args<S: Into<String>>(self, args: Vec<S>) -> MethodExpectation<T> {
        let expectation = Expectation::new(self.method.clone()).with_args(args);

        MethodExpectation {
            method: self.method,
            registry: self.registry,
            expectation,
            _marker: PhantomData,
        }
    }

    /// Set the arguments for the expectation using a formatted string
    pub fn with_args_fmt<S: fmt::Display>(self, args: &[S]) -> MethodExpectation<T> {
        let args_str: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();

        self.with_args(args_str)
    }

    /// Set no arguments for the expectation
    pub fn with_no_args(self) -> MethodExpectation<T> {
        let expectation = Expectation::new(self.method.clone());

        MethodExpectation {
            method: self.method,
            registry: self.registry,
            expectation,
            _marker: PhantomData,
        }
    }
}

impl<T> MethodExpectation<T> {
    /// Set the number of times the method is expected to be called
    pub fn times(mut self, times: ExpectedTimes) -> Self {
        self.expectation = self.expectation.times(times);
        self
    }

    /// The method is expected to be called exactly once
    pub fn once(self) -> Self {
        self.times(ExpectedTimes::Exact(1))
    }

    /// The method is expected to be called exactly twice
    pub fn twice(self) -> Self {
        self.times(ExpectedTimes::Exact(2))
    }

    /// The method is expected to be called exactly n times
    pub fn exactly(self, n: usize) -> Self {
        self.times(ExpectedTimes::Exact(n))
    }

    /// The method is expected to be called at least n times
    pub fn at_least(self, n: usize) -> Self {
        self.times(ExpectedTimes::AtLeast(n))
    }

    /// The method is expected to be called at most n times
    pub fn at_most(self, n: usize) -> Self {
        self.times(ExpectedTimes::AtMost(n))
    }

    /// The method is expected to be called any number of times
    pub fn any_times(self) -> Self {
        self.times(ExpectedTimes::Any)
    }

    /// The method is expected not to be called
    pub fn never(self) -> Self {
        self.times(ExpectedTimes::Exact(0))
    }

    /// Finalize the expectation and add it to the registry
    pub fn build(self) -> TestResult<()> {
        self.registry.expect(self.expectation)
    }
}

/// Common trait for mock objects that can be configured
pub trait MockConfig: Sized {
    /// The type of configuration for this mock
    type Config;

    /// Get the configuration for this mock
    fn config(&self) -> &Self::Config;
}

#[cfg(test)]
mod tests {
    use super::super::MockRegistry;
    use super::*;

    #[test]
    fn test_mock_config() {
        let registry = Arc::new(MockRegistry::new());
        let config = MockConfig::<()>::new(&registry);

        // Set up expectations
        config
            .expect("test_method")
            .with_args(vec!["arg1", "arg2"])
            .once()
            .build()
            .unwrap();

        // Record a call
        registry
            .record_call("test_method", &["arg1".to_string(), "arg2".to_string()])
            .unwrap();

        // Verify expectations
        config.verify().unwrap();
    }

    #[test]
    fn test_mock_config_with_args_fmt() {
        let registry = Arc::new(MockRegistry::new());
        let config = MockConfig::<()>::new(&registry);

        // Set up expectations
        config
            .expect("test_method")
            .with_args_fmt(&[42, "test"])
            .once()
            .build()
            .unwrap();

        // Record a call
        registry
            .record_call("test_method", &["42".to_string(), "test".to_string()])
            .unwrap();

        // Verify expectations
        config.verify().unwrap();
    }

    #[test]
    fn test_mock_config_times_variants() {
        let registry = Arc::new(MockRegistry::new());
        let config = MockConfig::<()>::new(&registry);

        // Set up expectations
        config
            .expect("method1")
            .with_no_args()
            .once()
            .build()
            .unwrap();

        config
            .expect("method2")
            .with_no_args()
            .twice()
            .build()
            .unwrap();

        config
            .expect("method3")
            .with_no_args()
            .exactly(3)
            .build()
            .unwrap();

        config
            .expect("method4")
            .with_no_args()
            .at_least(2)
            .build()
            .unwrap();

        config
            .expect("method5")
            .with_no_args()
            .at_most(2)
            .build()
            .unwrap();

        config
            .expect("method6")
            .with_no_args()
            .never()
            .build()
            .unwrap();

        // Record calls
        registry.record_call("method1", &[]).unwrap();

        registry.record_call("method2", &[]).unwrap();
        registry.record_call("method2", &[]).unwrap();

        registry.record_call("method3", &[]).unwrap();
        registry.record_call("method3", &[]).unwrap();
        registry.record_call("method3", &[]).unwrap();

        registry.record_call("method4", &[]).unwrap();
        registry.record_call("method4", &[]).unwrap();

        registry.record_call("method5", &[]).unwrap();

        // Verify expectations
        config.verify().unwrap();
    }
}
