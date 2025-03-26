/// Common test utilities and fixtures
///
/// This module provides common utilities used across tests, including:
/// - Test fixtures
/// - Test data generators
/// - Helper functions
/// - Test environment setup and cleanup
use anyhow::Result;
use std::sync::Arc;

/// Test context trait for test environments
pub trait TestContext {
    /// Setup the test context
    fn setup() -> Self;

    /// Cleanup the test context
    fn cleanup(&self);
}

/// Helper struct for generating random test data
#[derive(Debug, Default)]
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn random_string(length: usize) -> String {
        use rand::{Rng, distributions::Alphanumeric};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    pub fn random_number(min: i32, max: i32) -> i32 {
        use rand::Rng;
        rand::thread_rng().gen_range(min..=max)
    }
}

/// Run with a test context and automatically clean up
pub async fn with_test_context<T, F, Fut, R>(test_fn: F) -> Result<R>
where
    T: TestContext,
    F: FnOnce(Arc<T>) -> Fut,
    Fut: std::future::Future<Output = Result<R>>,
{
    let context = Arc::new(T::setup());
    let result = test_fn(context.clone()).await;
    context.cleanup();
    result
}
