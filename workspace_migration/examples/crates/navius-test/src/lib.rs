// Navius Test Framework
//
// This crate provides the testing infrastructure for the Navius framework,
// focusing on cross-crate integration testing.

pub mod error;
pub mod fixture;
pub mod harness;
pub mod mock;

pub use error::TestError;
pub use fixture::TestFixture;

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

/// Assert that a result is Ok and optionally check its value
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        assert!(
            $expr.is_ok(),
            "Expected Ok but got Err: {:?}",
            $expr.err().unwrap()
        );
    };
    ($expr:expr, $expected:expr) => {
        match $expr {
            Ok(val) => assert_eq!(val, $expected),
            Err(e) => panic!("Expected Ok({:?}) but got Err: {:?}", $expected, e),
        }
    };
}

/// Assert that a result is Err and optionally check the error
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        assert!(
            $expr.is_err(),
            "Expected Err but got Ok: {:?}",
            $expr.unwrap()
        );
    };
    ($expr:expr, $error_type:path) => {
        match $expr {
            Ok(val) => panic!("Expected Err but got Ok: {:?}", val),
            Err(e) => assert!(
                std::any::Any::type_id(&e) == std::any::Any::type_id(&$error_type(())),
                "Expected error of type {} but got {:?}",
                stringify!($error_type),
                e
            ),
        }
    };
}
