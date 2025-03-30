use crate::error::{TestError, TestResult};
use crate::fixture::TestFixture;
use crate::mock::MockRegistry;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// A test harness for running multi-crate tests
pub struct TestHarness {
    /// The test fixture
    fixture: TestFixture,

    /// The mock registry
    mock_registry: MockRegistry,

    /// The tokio runtime for async tests
    runtime: Option<Runtime>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self {
            fixture: TestFixture::new().build(),
            mock_registry: MockRegistry::new(),
            runtime: None,
        }
    }

    /// Create a new test harness with an existing fixture
    pub fn with_fixture(fixture: TestFixture) -> Self {
        Self {
            fixture,
            mock_registry: MockRegistry::new(),
            runtime: None,
        }
    }

    /// Create a new test harness with an existing mock registry
    pub fn with_mock_registry(mock_registry: MockRegistry) -> Self {
        Self {
            fixture: TestFixture::new().build(),
            mock_registry,
            runtime: None,
        }
    }

    /// Get the test fixture
    pub fn fixture(&self) -> &TestFixture {
        &self.fixture
    }

    /// Get the mock registry
    pub fn mock_registry(&self) -> &MockRegistry {
        &self.mock_registry
    }

    /// Initialize a tokio runtime for async tests
    pub fn init_runtime(&mut self) -> TestResult<()> {
        let runtime = Runtime::new().map_err(|e| {
            TestError::setup_error(format!("Failed to create tokio runtime: {}", e))
        })?;

        self.runtime = Some(runtime);
        Ok(())
    }

    /// Run an async function in the test harness
    pub fn run_async<F, Fut, T>(&self, f: F) -> TestResult<T>
    where
        F: FnOnce(Arc<TestFixture>, Arc<MockRegistry>) -> Fut,
        Fut: Future<Output = TestResult<T>>,
    {
        let runtime = self.runtime.as_ref().ok_or_else(|| {
            TestError::setup_error("Tokio runtime not initialized. Call init_runtime() first.")
        })?;

        // Create Arc wrappers for the fixture and mock registry
        let fixture = Arc::new(self.fixture.clone());
        let mock_registry = Arc::new(self.mock_registry.clone());

        // Run the function in the tokio runtime
        runtime.block_on(async { f(fixture, mock_registry).await })
    }

    /// Run a sync function in the test harness
    pub fn run<F, T>(&self, f: F) -> TestResult<T>
    where
        F: FnOnce(&TestFixture, &MockRegistry) -> TestResult<T>,
    {
        f(&self.fixture, &self.mock_registry)
    }

    /// Tear down the test harness
    pub fn tear_down(self) -> TestResult<()> {
        // Tear down the fixture
        self.fixture.tear_down()?;

        // Clear the mock registry
        self.mock_registry.clear()?;

        // Drop the runtime
        if let Some(runtime) = self.runtime {
            drop(runtime);
        }

        Ok(())
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Attempt to tear down the fixture
        let _ = self.fixture.tear_down();

        // Attempt to clear the mock registry
        let _ = self.mock_registry.clear();
    }
}

/// Builder for test harnesses
pub struct TestHarnessBuilder {
    /// The test fixture builder
    fixture_builder: Option<TestFixture>,

    /// The mock registry
    mock_registry: Option<MockRegistry>,

    /// Whether to initialize a tokio runtime
    init_runtime: bool,
}

impl TestHarnessBuilder {
    /// Create a new test harness builder
    pub fn new() -> Self {
        Self {
            fixture_builder: None,
            mock_registry: None,
            init_runtime: false,
        }
    }

    /// Set the test fixture
    pub fn with_fixture(mut self, fixture: TestFixture) -> Self {
        self.fixture_builder = Some(fixture);
        self
    }

    /// Set the mock registry
    pub fn with_mock_registry(mut self, mock_registry: MockRegistry) -> Self {
        self.mock_registry = Some(mock_registry);
        self
    }

    /// Initialize a tokio runtime
    pub fn with_runtime(mut self) -> Self {
        self.init_runtime = true;
        self
    }

    /// Build the test harness
    pub fn build(self) -> TestResult<TestHarness> {
        let fixture = self
            .fixture_builder
            .unwrap_or_else(|| TestFixture::new().build());
        let mock_registry = self.mock_registry.unwrap_or_else(MockRegistry::new);

        let mut harness = TestHarness {
            fixture,
            mock_registry,
            runtime: None,
        };

        if self.init_runtime {
            harness.init_runtime()?;
        }

        Ok(harness)
    }
}

impl Default for TestHarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple test case that can be executed by the test harness
pub struct TestCase<F, T> {
    /// The name of the test case
    name: String,

    /// The test function
    test_fn: F,

    /// Whether the test is async
    is_async: bool,

    /// Marker for the return type
    _marker: std::marker::PhantomData<T>,
}

impl<F, T> TestCase<F, T> {
    /// Create a new sync test case
    pub fn new<S: Into<String>>(name: S, test_fn: F) -> Self
    where
        F: FnOnce(&TestFixture, &MockRegistry) -> TestResult<T>,
    {
        Self {
            name: name.into(),
            test_fn,
            is_async: false,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the name of the test case
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if the test is async
    pub fn is_async(&self) -> bool {
        self.is_async
    }
}

impl<F, Fut, T> TestCase<F, T>
where
    F: FnOnce(Arc<TestFixture>, Arc<MockRegistry>) -> Fut,
    Fut: Future<Output = TestResult<T>>,
{
    /// Create a new async test case
    pub fn new_async<S: Into<String>>(name: S, test_fn: F) -> TestCase<F, T> {
        TestCase {
            name: name.into(),
            test_fn,
            is_async: true,
            _marker: std::marker::PhantomData,
        }
    }
}
