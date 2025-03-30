// Navius Test Utilities
//
// This crate provides common testing utilities for the Navius framework.

use std::sync::Arc;
use thiserror::Error;

/// Error type for test utilities
#[derive(Error, Debug)]
pub enum TestError {
    /// General error
    #[error("Test error: {0}")]
    General(String),

    /// Initialization error
    #[error("Test initialization error: {0}")]
    Initialization(String),

    /// Validation error
    #[error("Test validation error: {0}")]
    Validation(String),
}

/// Result type for test utilities
pub type Result<T> = std::result::Result<T, TestError>;

/// Test context for setting up test environments
pub struct TestContext {
    name: String,
    teardown_hooks: Vec<Box<dyn FnOnce() -> Result<()> + Send>>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            teardown_hooks: Vec::new(),
        }
    }

    /// Add a teardown hook
    pub fn with_teardown<F>(&mut self, hook: F) -> &mut Self
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        self.teardown_hooks.push(Box::new(hook));
        self
    }

    /// Get the test name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Run teardown hooks in reverse order
        while let Some(hook) = self.teardown_hooks.pop() {
            if let Err(e) = hook() {
                eprintln!("Error during test teardown: {}", e);
            }
        }
    }
}

/// Mock factory for creating test doubles
pub struct MockFactory;

impl MockFactory {
    /// Create a new mock factory
    pub fn new() -> Self {
        Self
    }

    /// Create a simple mock that returns the specified value
    pub fn create_value_mock<T: Clone + 'static>(
        &self,
        value: T,
    ) -> Arc<dyn Fn() -> T + Send + Sync> {
        let mock_fn = move || value.clone();
        Arc::new(mock_fn)
    }
}
