# Integration Test Utilities

## Overview

The Integration Test Utilities provide a comprehensive framework for creating, configuring, and running tests that span multiple crates in the Navius workspace. This component builds upon the Mock Interface Registry, Test Fixture, and Test Harness components to provide a higher-level abstraction for integration testing.

## Key Components

### 1. Integration Context

The `IntegrationContext` serves as the central environment for integration tests, providing:

- Management of test fixtures that contain registered components
- Access to a shared mock registry for setting expectations
- Environment variable management for tests
- Test resource management (files, directories, etc.)
- Automatic cleanup of test resources

```rust
pub struct IntegrationContext {
    config: IntegrationTestConfig,
    fixtures: Vec<Arc<TestFixture>>,
    test_dir: Option<PathBuf>,
    env_vars: RwLock<HashMap<String, String>>,
    registry: Arc<MockRegistry>,
    original_env: HashMap<String, Option<String>>,
}
```

### 2. Test Configuration

The `TestConfig` system provides a flexible way to configure integration tests:

- Test resources configuration (input files, output directories)
- Mock configuration (default values, error simulation)
- Environment variables
- Timeout settings for tests and operations
- Custom options for specific test scenarios

Configuration can be loaded from JSON or TOML files, or created programmatically using the builder pattern:

```rust
let config = TestConfigBuilder::new("api_tests")
    .with_description("API integration tests")
    .with_resource_dir("./test_resources")
    .with_test_timeout(60)
    .with_env_var("DATABASE_URL", "postgres://localhost/test")
    .build();
```

### 3. Test Runner

The `TestRunner` provides utilities for running integration tests and managing test execution:

- Running single tests or test suites
- Timeout management for tests
- Collection of test results in structured reports
- Automatic cleanup of test resources
- Configurable output for test results

```rust
pub struct TestRunner {
    config: TestConfig,
    report_dir: Option<PathBuf>,
    tests: Vec<Box<dyn IntegrationTest>>,
    fail_fast: bool,
    verbose: bool,
}
```

### 4. Cross-Crate Testing

The `CrossCrateTestBuilder` provides specific utilities for tests that span multiple crates:

- Specifying which crates are involved in a test
- Managing dependencies between crates
- Setting up appropriate test environments for multi-crate tests
- Configuring resources specific to cross-crate tests

```rust
let builder = CrossCrateTestBuilder::new("auth_service_test")
    .with_crate("navius-core")
    .with_crate("navius-auth")
    .with_crate("navius-db")
    .with_timeout(Duration::from_secs(30))
    .with_env_var("AUTH_PROVIDER", "mock");
```

## API Overview

### Integration Context API

The `IntegrationContext` provides a simple yet powerful API for setting up and managing test environments:

```rust
impl IntegrationContext {
    pub fn new(config: IntegrationTestConfig) -> TestResult<Self>;
    pub fn create_fixture(&mut self) -> TestResult<Arc<TestFixture>>;
    pub fn test_dir(&self) -> TestResult<PathBuf>;
    pub fn set_env_var(&self, key: &str, value: &str) -> TestResult<()>;
    pub fn get_env_var(&self, key: &str) -> TestResult<Option<String>>;
    pub fn registry(&self) -> Arc<MockRegistry>;
    pub fn create_harness(&self) -> TestHarness;
}
```

### Test Runner API

The `TestRunner` provides a comprehensive API for executing tests and reporting results:

```rust
impl TestRunner {
    pub fn new(config: TestConfig) -> Self;
    pub fn with_report_dir(self, dir: impl Into<PathBuf>) -> Self;
    pub fn with_test(self, test: impl IntegrationTest + 'static) -> Self;
    pub fn with_fail_fast(self, fail_fast: bool) -> Self;
    pub fn with_verbose(self, verbose: bool) -> Self;
    pub fn run_all(&self) -> TestResult<Vec<TestReport>>;
    pub fn run_test(&self, test: &dyn IntegrationTest) -> TestResult<TestReport>;
}
```

### Macros for Test Creation

The integration test utilities include macros for easier test creation:

```rust
// Create a test case
let test = test_case!("user_service_test", |context| {
    // Test implementation
    Ok(())
});

// Create a test case with setup and cleanup
let test = test_case!("database_test", |context| {
    // Test implementation
    Ok(())
}, setup: |context| {
    // Setup code
    Ok(())
}, cleanup: |context| {
    // Cleanup code
    Ok(())
});

// Create a test suite
let runner = test_suite!("api_tests", config,
    test_case!("test1", |ctx| { /* ... */ Ok(()) }),
    test_case!("test2", |ctx| { /* ... */ Ok(()) }),
);
```

## Integration with Test Fixture and Mock Registry

The Integration Test Utilities seamlessly integrate with the TestFixture and MockRegistry systems:

```rust
test_case!("user_repository_test", |context| {
    // Create a fixture
    let mut ctx = context.to_owned();
    let fixture = ctx.create_fixture()?;
    
    // Get the mock registry
    let registry = context.registry();
    
    // Create and register a mock database
    let mock_db = MockDatabaseClient::new(registry.clone());
    fixture.register_component::<dyn DatabaseClient>(mock_db.clone())?;
    
    // Set up expectations
    registry.expect("DatabaseClient", "query")
        .with(("SELECT * FROM users WHERE id = ?", vec![Value::Integer(1)]))
        .returns(Ok(vec![create_user_row(1, "Test User", "test@example.com")]));
    
    // Create the repository with the mock database
    let repo = UserRepository::new(fixture.get_component::<dyn DatabaseClient>()?);
    
    // Test the repository
    let user = repo.get_user_by_id(1)?;
    
    // Verify the result
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Test User");
    
    Ok(())
});
```

## Configuration Examples

### Basic Test Configuration

```rust
let config = TestConfigBuilder::new("basic_tests")
    .with_description("Basic integration tests")
    .with_test_timeout(30)
    .build();
```

### Full Test Configuration

```rust
let config = TestConfigBuilder::new("comprehensive_tests")
    .with_description("Comprehensive integration tests")
    .with_resource_dir("/tmp/test_resources")
    .with_output_dir("/tmp/test_output")
    .with_input_file("users", "/tmp/test_data/users.json")
    .with_cleanup(true)
    .with_verify_expectations(true)
    .with_env_var("APP_ENV", "test")
    .with_env_var("DATABASE_URL", "postgres://localhost/test")
    .with_test_timeout(60)
    .with_operation_timeout(10)
    .with_connection_timeout(5)
    .with_option("log_level", "debug")
    .build();
```

## Cross-Crate Test Example

```rust
// Create a cross-crate test
let builder = CrossCrateTestBuilder::new("auth_api_test")
    .with_crate("navius-core")
    .with_crate("navius-auth")
    .with_crate("navius-http")
    .with_timeout(Duration::from_secs(30))
    .with_env_var("AUTH_PROVIDER", "mock");

let runner = builder.build()?;

let result = runner.run(|context| {
    // Test the auth API with components from multiple crates
    let registry = context.registry();
    
    // Set up expectations for the auth provider
    registry.expect("AuthProvider", "authenticate")
        .with(("username", "password"))
        .returns(Ok(AuthToken::new("user123", "token123")));
    
    // Test the API
    let auth_api = AuthApi::new(context.get_component()?);
    let token = auth_api.login("username", "password")?;
    
    assert_eq!(token.user_id, "user123");
    assert_eq!(token.token, "token123");
    
    Ok(())
})?;
```

## Next Steps

With the Integration Test Utilities complete, the next steps in the Navius workspace migration include:

1. **Update existing tests** to use the new cross-crate testing infrastructure
2. **Create comprehensive documentation** for the testing framework to guide developers
3. **Develop additional mock interfaces** for specific Navius components
4. **Integrate the testing framework** with the build system for automated testing

The Integration Test Utilities, together with the Mock Interface Registry and Error Testing Framework, form a complete testing infrastructure for the Navius application, enabling comprehensive testing of components across crate boundaries. 