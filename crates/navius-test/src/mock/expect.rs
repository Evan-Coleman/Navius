use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::error::TestResult;
use crate::mock::MockRegistry;
use crate::mock::config::{Expectation, ExpectedTimes};

/// A trait for setting up expectations on mock objects
pub trait MockExpect {
    /// The type of the return value
    type ReturnType;

    /// Set up an expectation for a method call
    fn expect_call<S: Into<String>>(
        &self,
        method: S,
        args: &[String],
        times: ExpectedTimes,
        return_value: Self::ReturnType,
    ) -> TestResult<()>;
}

/// A container for expected return values
#[derive(Debug)]
pub struct ExpectedReturn<T> {
    /// The values to return
    pub values: Mutex<Vec<T>>,
}

impl<T> ExpectedReturn<T> {
    /// Create a new container for expected return values
    pub fn new() -> Self {
        Self {
            values: Mutex::new(Vec::new()),
        }
    }

    /// Add a value to return
    pub fn add(&self, value: T) {
        let mut values = self.values.lock().unwrap();
        values.push(value);
    }

    /// Get the next value to return
    pub fn next(&self) -> Option<T>
    where
        T: Clone,
    {
        let mut values = self.values.lock().unwrap();
        if values.is_empty() {
            None
        } else if values.len() == 1 {
            // If there's only one value, keep returning it
            Some(values[0].clone())
        } else {
            // Otherwise, remove and return the first value
            Some(values.remove(0))
        }
    }
}

/// Trait for accessing expected returns
pub trait HasExpectedReturns<T> {
    /// Get the expected returns for a method
    fn expected_returns(&self, method: &str) -> Option<&ExpectedReturn<T>>;
}

/// A builder for setting up method expectations
#[derive(Debug)]
pub struct MethodExpectBuilder<'a, T> {
    /// The registry to add the expectation to
    registry: &'a MockRegistry,

    /// The name of the mock object
    mock_name: String,

    /// The name of the method
    method: String,

    /// The expected arguments
    args: Vec<String>,

    /// The number of times the method is expected to be called
    times: ExpectedTimes,

    /// Type marker
    _marker: PhantomData<T>,
}

impl<'a, T> MethodExpectBuilder<'a, T> {
    /// Create a new method expectation builder
    pub fn new<S: Into<String>>(registry: &'a MockRegistry, mock_name: S, method: S) -> Self {
        Self {
            registry,
            mock_name: mock_name.into(),
            method: method.into(),
            args: Vec::new(),
            times: ExpectedTimes::Any,
            _marker: PhantomData,
        }
    }

    /// Set the expected arguments
    pub fn with_args<S: Into<String>>(mut self, args: Vec<S>) -> Self {
        self.args = args.into_iter().map(|a| a.into()).collect();
        self
    }

    /// Set the number of times the method is expected to be called
    pub fn times(mut self, times: ExpectedTimes) -> Self {
        self.times = times;
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

    /// The method is expected not to be called
    pub fn never(self) -> Self {
        self.times(ExpectedTimes::Exact(0))
    }

    /// Build the expectation and add it to the registry
    pub fn build(self) -> TestResult<()> {
        let expectation = Expectation::new(self.method)
            .with_args(self.args)
            .times(self.times);

        self.registry.add_expectation(&self.mock_name, expectation)
    }

    /// Set the return value for the method and build the expectation
    pub fn returns<R: 'static + Clone + Send + Sync>(self, return_value: R) -> TestResult<()> {
        let expectation = Expectation::new(self.method)
            .with_args(self.args)
            .times(self.times)
            .with_return(Box::new(return_value));

        self.registry.add_expectation(&self.mock_name, expectation)
    }
}

/// A builder for setting up mock expectations
#[derive(Debug)]
pub struct MockExpectBuilder<'a, T> {
    /// The registry to add the expectation to
    registry: &'a MockRegistry,

    /// The name of the mock object
    mock_name: String,

    /// Type marker
    _marker: PhantomData<T>,
}

impl<'a, T> MockExpectBuilder<'a, T> {
    /// Create a new mock expectation builder
    pub fn new<S: Into<String>>(registry: &'a MockRegistry, mock_name: S) -> Self {
        Self {
            registry,
            mock_name: mock_name.into(),
            _marker: PhantomData,
        }
    }

    /// Set up an expectation for a method call
    pub fn method<S: Into<String>>(&self, method: S) -> MethodExpectBuilder<'a, T> {
        let method_str = method.into();
        MethodExpectBuilder::new(self.registry, &self.mock_name, &method_str)
    }
}

/// A trait for verifying expectations on mock objects
pub trait MockVerify {
    /// Verify that all expectations have been met
    fn verify(&self) -> TestResult<()>;
}

/// Macro for building expectations
#[macro_export]
macro_rules! expect {
    ($mock:expr, $method:expr) => {
        $mock.expect($method)
    };

    ($mock:expr, $method:expr, $($arg:expr),*) => {
        $mock.expect($method).with_args(vec![$($arg),*])
    };

    ($mock:expr, $method:expr, $($arg:expr),* ; $times:expr) => {
        $mock.expect($method).with_args(vec![$($arg),*]).times($times)
    };

    ($mock:expr, $method:expr, $($arg:expr),* => $return:expr) => {
        $mock.expect($method).with_args(vec![$($arg),*]).returns($return)
    };

    ($mock:expr, $method:expr, $($arg:expr),* ; $times:expr => $return:expr) => {
        $mock.expect($method).with_args(vec![$($arg),*]).times($times).returns($return)
    };
}

/// Macros for common expectation patterns
#[macro_export]
macro_rules! expect_once {
    ($mock:expr, $method:expr) => {
        $mock.expect($method).once()
    };

    ($mock:expr, $method:expr, $($arg:expr),*) => {
        $mock.expect($method).with_args(vec![$($arg),*]).once()
    };

    ($mock:expr, $method:expr, $($arg:expr),* => $return:expr) => {
        $mock.expect($method).with_args(vec![$($arg),*]).once().returns($return)
    };
}

/// Macros for common expectation patterns
#[macro_export]
macro_rules! expect_never {
    ($mock:expr, $method:expr) => {
        $mock.expect($method).never()
    };

    ($mock:expr, $method:expr, $($arg:expr),*) => {
        $mock.expect($method).with_args(vec![$($arg),*]).never()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_return() {
        let expected = ExpectedReturn::<i32>::new();
        expected.add(42);

        assert_eq!(expected.next(), Some(42));
        assert_eq!(expected.next(), Some(42)); // Should keep returning the last value

        let expected = ExpectedReturn::<String>::new();
        expected.add("first".to_string());
        expected.add("second".to_string());
        expected.add("third".to_string());

        assert_eq!(expected.next(), Some("first".to_string()));
        assert_eq!(expected.next(), Some("second".to_string()));
        assert_eq!(expected.next(), Some("third".to_string()));
        assert_eq!(expected.next(), Some("third".to_string())); // Should keep returning the last value
    }

    #[test]
    fn test_method_expect_builder() {
        let registry = MockRegistry::new();

        let builder = MethodExpectBuilder::<(), i32>::new(&registry, "TestMock", "test_method");

        let builder = builder.with_args(vec!["arg1", "arg2"]).once();

        builder.build().unwrap();

        // Verify that the expectation was added
        let expectations = registry.verify();
        assert!(expectations.is_err()); // Will fail because the method wasn't called
    }

    #[test]
    fn test_mock_expect_builder() {
        let registry = MockRegistry::new();

        let builder = MockExpectBuilder::<(), i32>::new(&registry, "TestMock");

        builder
            .method("test_method")
            .with_args(vec!["arg1", "arg2"])
            .once()
            .build()
            .unwrap();

        // Record a call
        registry
            .record_call("test_method", &["arg1".to_string(), "arg2".to_string()])
            .unwrap();

        // Verify that the expectation was met
        registry.verify().unwrap();
    }
}
