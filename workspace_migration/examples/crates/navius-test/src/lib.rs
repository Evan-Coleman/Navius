// Navius Test Framework
//
// This crate provides the testing infrastructure for the Navius framework,
// focusing on cross-crate integration testing.

pub mod error;
pub use error::{
    ErrorContext, ErrorInjection, ErrorPropagationTracker, ErrorVerifier, TestError, TestResult,
    simulate_error,
};

pub mod fixture;
pub use fixture::{Resource, ResourceCleanup, TestFixture, TestFixtureBuilder};

pub mod harness;
pub use harness::{TestHarness, TestHarnessBuilder};

pub mod mock;
pub use mock::{Expectation, ExpectationBuilder, MockRegistry, VerifyMode};

/// Re-export common testing utilities
pub mod prelude {
    pub use crate::error::{TestError, TestResult};
    pub use crate::fixture::{TestFixture, TestFixtureBuilder};
    pub use crate::harness::TestHarness;
    pub use crate::mock::MockRegistry;

    // Re-export commonly used testing libraries
    pub use assert_matches;
    pub use mockall;

    // Re-export testing macros
    pub use crate::assert_err;
    pub use crate::assert_ok;
}

/// Macro to assert that a result is Ok and return the unwrapped value
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                panic!("Expected Ok, got Err: {:?}", err);
            }
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                panic!("{}: {:?}", $msg, err);
            }
        }
    };
}

/// Macro to assert that a result is Err and return the unwrapped error
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(val) => {
                panic!("Expected Err, got Ok: {:?}", val);
            }
            Err(err) => err,
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => {
                panic!("{}: {:?}", $msg, val);
            }
            Err(err) => err,
        }
    };
}

/// Macro to assert that a result is Err and contains a specific error variant
#[macro_export]
macro_rules! assert_err_variant {
    ($expr:expr, $pattern:pat) => {
        match $expr {
            Ok(val) => {
                panic!("Expected Err, got Ok: {:?}", val);
            }
            Err($pattern) => {}
            Err(err) => {
                panic!(
                    "Expected Err variant {:?}, got Err: {:?}",
                    stringify!($pattern),
                    err
                );
            }
        }
    };
}

/// Macro to assert that an error is injected
#[macro_export]
macro_rules! assert_injected_error {
    ($expr:expr, $message:expr) => {
        match $expr {
            Ok(val) => {
                panic!("Expected error to be injected, got Ok: {:?}", val);
            }
            Err(err) => {
                assert!(
                    err.to_string().contains($message),
                    "Expected error message to contain '{}', got '{}'",
                    $message,
                    err
                );
            }
        }
    };
}

/// Macro to verify error propagation
#[macro_export]
macro_rules! verify_error_path {
    ($tracker:expr, $($component:expr),+) => {
        $(
            assert!(
                $tracker.passed_through($component),
                "Expected error to pass through {}, but it didn't",
                $component
            );
        )+
    };
}

/// Macro to verify error context
#[macro_export]
macro_rules! verify_error_context {
    ($tracker:expr, $key:expr, $value:expr) => {
        match $tracker.get_context($key) {
            Some(val) => {
                assert_eq!(
                    val, $value,
                    "Expected context value '{}' for key '{}', got '{}'",
                    $value, $key, val
                );
            }
            None => {
                panic!(
                    "Expected error to have context key '{}', but it didn't",
                    $key
                );
            }
        }
    };
    ($tracker:expr, $key:expr) => {
        assert!(
            $tracker.has_context($key),
            "Expected error to have context key '{}', but it didn't",
            $key
        );
    };
}
