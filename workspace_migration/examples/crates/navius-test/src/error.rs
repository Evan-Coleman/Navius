use std::fmt;
use thiserror::Error;

/// Result type for the test framework
pub type TestResult<T> = Result<T, TestError>;

/// Error type for the test framework
#[derive(Error, Debug)]
pub enum TestError {
    /// Error during test fixture setup
    #[error("Failed to set up test fixture: {0}")]
    SetupError(String),

    /// Error during test fixture teardown
    #[error("Failed to tear down test fixture: {0}")]
    TeardownError(String),

    /// Error when a required component is missing
    #[error("Required test component not found: {0}")]
    MissingComponent(String),

    /// Error when a mock implementation is not registered
    #[error("Mock implementation not registered for interface: {0}")]
    MockNotRegistered(String),

    /// Error when a test configuration is invalid
    #[error("Invalid test configuration: {0}")]
    InvalidConfiguration(String),

    /// Error during resource allocation
    #[error("Failed to allocate test resource: {0}")]
    ResourceAllocationError(String),

    /// Error when a test assertion fails
    #[error("Test assertion failed: {0}")]
    AssertionError(String),

    /// Error from an external service or dependency
    #[error("External dependency error: {0}")]
    ExternalError(String),

    /// IO Error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serde JSON Error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Unknown error
    #[error("Unknown test error: {0}")]
    Unknown(String),
}

impl TestError {
    /// Create a new setup error
    pub fn setup_error<S: fmt::Display>(msg: S) -> Self {
        Self::SetupError(msg.to_string())
    }

    /// Create a new teardown error
    pub fn teardown_error<S: fmt::Display>(msg: S) -> Self {
        Self::TeardownError(msg.to_string())
    }

    /// Create a new missing component error
    pub fn missing_component<S: fmt::Display>(component: S) -> Self {
        Self::MissingComponent(component.to_string())
    }

    /// Create a new mock not registered error
    pub fn mock_not_registered<S: fmt::Display>(interface: S) -> Self {
        Self::MockNotRegistered(interface.to_string())
    }

    /// Create a new invalid configuration error
    pub fn invalid_configuration<S: fmt::Display>(msg: S) -> Self {
        Self::InvalidConfiguration(msg.to_string())
    }

    /// Create a new resource allocation error
    pub fn resource_allocation_error<S: fmt::Display>(msg: S) -> Self {
        Self::ResourceAllocationError(msg.to_string())
    }

    /// Create a new assertion error
    pub fn assertion_error<S: fmt::Display>(msg: S) -> Self {
        Self::AssertionError(msg.to_string())
    }

    /// Create a new external error
    pub fn external_error<S: fmt::Display>(msg: S) -> Self {
        Self::ExternalError(msg.to_string())
    }

    /// Create a new unknown error
    pub fn unknown<S: fmt::Display>(msg: S) -> Self {
        Self::Unknown(msg.to_string())
    }
}

/// Helper trait for converting errors to TestError
pub trait IntoTestError {
    /// Convert this error to a TestError
    fn into_test_error(self) -> TestError;
}

impl<E: std::error::Error + 'static> IntoTestError for E {
    fn into_test_error(self) -> TestError {
        if let Some(err) = std::any::Any::downcast_ref::<std::io::Error>(&self) {
            return TestError::IoError(std::io::Error::new(err.kind(), self));
        }

        if let Some(err) = std::any::Any::downcast_ref::<serde_json::Error>(&self) {
            return TestError::JsonError(serde_json::Error::custom(self));
        }

        TestError::Unknown(self.to_string())
    }
}
