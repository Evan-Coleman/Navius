use crate::error::{TestError, TestResult};
use crate::fixture::TestFixture;
use crate::mock::MockRegistry;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};

/// A test harness for running tests with fixtures and mocks
pub struct TestHarness {
    /// The test fixture
    fixture: TestFixture,

    /// The mock registry
    mock_registry: MockRegistry,

    /// The tokio runtime for async tests
    runtime: Option<Runtime>,
}

impl TestHarness {
    /// Create a new test harness without a runtime
    pub fn new() -> Self {
        Self {
            fixture: TestFixture::new().build(),
            mock_registry: MockRegistry::new(),
            runtime: None,
        }
    }

    /// Create a new test harness with a tokio runtime
    pub fn with_runtime() -> TestResult<Self> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                TestError::setup_error(format!("Failed to create tokio runtime: {}", e))
            })?;

        Ok(Self {
            fixture: TestFixture::new().build(),
            mock_registry: MockRegistry::new(),
            runtime: Some(runtime),
        })
    }

    /// Get the test fixture
    pub fn fixture(&self) -> &TestFixture {
        &self.fixture
    }

    /// Get the mock registry
    pub fn mock_registry(&self) -> &MockRegistry {
        &self.mock_registry
    }

    /// Run a synchronous test function
    pub fn run<F, T>(&self, f: F) -> TestResult<T>
    where
        F: FnOnce(&TestFixture, &MockRegistry) -> TestResult<T>,
    {
        // Run the test function
        let result = f(&self.fixture, &self.mock_registry);

        // Verify all mock expectations
        self.mock_registry.verify()?;

        // Return the test result
        result
    }

    /// Run an asynchronous test function
    pub fn run_async<F, Fut, T>(&self, f: F) -> TestResult<T>
    where
        F: FnOnce(Arc<TestFixture>, Arc<MockRegistry>) -> Fut,
        Fut: Future<Output = TestResult<T>>,
    {
        // Get the runtime
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| TestError::setup_error("No runtime available for async test"))?;

        // Create shareable versions of the fixture and registry
        let fixture = Arc::new(self.fixture.clone());
        let registry = Arc::new(self.mock_registry.clone());

        // Run the async test function
        let result = runtime.block_on(f(fixture, registry));

        // Verify all mock expectations
        self.mock_registry.verify()?;

        // Return the test result
        result
    }

    /// Clean up all resources
    pub fn cleanup(&self) -> TestResult<()> {
        // Clean up the fixture
        self.fixture.cleanup()?;

        // Reset the mock registry
        self.mock_registry.reset()?;

        Ok(())
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Attempt to clean up resources
        let _ = self.cleanup();

        // Shutdown the runtime if it exists
        if let Some(runtime) = self.runtime.take() {
            drop(runtime);
        }
    }
}

/// Builder for creating test harnesses
pub struct TestHarnessBuilder {
    /// Whether to create a runtime
    with_runtime: bool,

    /// The fixture builder
    fixture_builder: Option<crate::fixture::TestFixtureBuilder>,
}

impl TestHarnessBuilder {
    /// Create a new test harness builder
    pub fn new() -> Self {
        Self {
            with_runtime: false,
            fixture_builder: None,
        }
    }

    /// Add a runtime to the harness
    pub fn with_runtime(mut self) -> Self {
        self.with_runtime = true;
        self
    }

    /// Set the fixture builder
    pub fn with_fixture(mut self, fixture_builder: crate::fixture::TestFixtureBuilder) -> Self {
        self.fixture_builder = Some(fixture_builder);
        self
    }

    /// Build the test harness
    pub fn build(self) -> TestResult<TestHarness> {
        // Create the fixture
        let fixture = match self.fixture_builder {
            Some(builder) => builder.build(),
            None => TestFixture::new().build(),
        };

        // Create the mock registry
        let mock_registry = MockRegistry::new();

        // Create the runtime if needed
        let runtime = if self.with_runtime {
            Some(
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| {
                        TestError::setup_error(format!("Failed to create tokio runtime: {}", e))
                    })?,
            )
        } else {
            None
        };

        Ok(TestHarness {
            fixture,
            mock_registry,
            runtime,
        })
    }
}

impl Default for TestHarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_harness() {
        // Create a harness
        let harness = TestHarnessBuilder::new().build().unwrap();

        // Run a test
        let result = harness.run(|fixture, _| {
            // Register a component
            fixture.register(42)?;

            // Get the component
            let value: i32 = fixture.get()?;

            // Assert the value
            assert_eq!(value, 42);

            Ok(value)
        });

        // Verify the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_async_harness() {
        // Create a harness with runtime
        let harness = TestHarnessBuilder::new().with_runtime().build().unwrap();

        // Run an async test
        let result = harness.run_async(|fixture, _| async move {
            // Register a component
            fixture.register(42)?;

            // Get the component
            let value: i32 = fixture.get()?;

            // Assert the value
            assert_eq!(value, 42);

            Ok(value)
        });

        // Verify the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    // Define a simple trait for testing
    trait TestService: Send + Sync {
        fn get_value(&self) -> i32;
    }

    // Define a mock implementation
    #[derive(Clone)]
    struct MockTestService {
        value: i32,
    }

    impl TestService for MockTestService {
        fn get_value(&self) -> i32 {
            self.value
        }
    }

    #[test]
    fn test_harness_with_mocks() {
        // Create a harness
        let harness = TestHarnessBuilder::new().build().unwrap();

        // Run a test with mocks
        let result = harness.run(|_, registry| {
            // Register a mock
            let mock = MockTestService { value: 42 };
            registry.register::<dyn TestService, MockTestService>(mock)?;

            // Set up an expectation
            registry.expect::<dyn TestService>("get_value")?;

            // Get the mock
            let mock: MockTestService = registry.get::<dyn TestService, MockTestService>()?;

            // Use the mock
            let value = mock.get_value();
            registry.record_call::<dyn TestService>("get_value")?;

            // Assert the value
            assert_eq!(value, 42);

            Ok(value)
        });

        // Verify the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
