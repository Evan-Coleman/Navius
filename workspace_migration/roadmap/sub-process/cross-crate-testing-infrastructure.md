# Cross-Crate Testing Infrastructure

**Status:** Planning Complete, Initial Prototype Created  
**Target Start Date:** April 12, 2025  
**Target Completion Date:** April 26, 2025  
**Last Updated:** March 29, 2025

## Overview

The Cross-Crate Testing Infrastructure will provide utilities, fixtures, and patterns for testing interactions between different crates in the Navius workspace. This is a critical component for ensuring that the interfaces between crates function as expected and that integration scenarios are thoroughly tested.

## Current Progress

- ✅ Design document completed with detailed implementation approach
- ✅ Key components identified and documented
- ✅ Testing strategies established
- ✅ Initial prototype implementation created with core components:
  - ✅ TestFixture: Component registration and resource management
  - ✅ MockRegistry: Interface mock registration and retrieval
  - ✅ TestHarness: Test environment management for both sync and async tests
  - ✅ Error handling utilities

## Objectives

1. Create a consistent approach to cross-crate testing
2. Provide utilities to simplify test setup for multi-crate scenarios
3. Enable comprehensive testing of interfaces between crates
4. Support mocking of dependencies for isolated testing
5. Establish patterns for testing complex integration scenarios
6. Ensure test coverage for error propagation across crate boundaries

## Key Components

### 1. Test Fixture Framework

A shared test fixture framework that provides:

- Standardized test setup and teardown
- Managed test resources (databases, caches, etc.)
- Configuration utilities for test environments
- Logging and diagnostic capabilities

### 2. Mock Implementation Registry

A registry of mock implementations for core interfaces:

- Mock database implementations
- Mock cache implementations
- Mock HTTP clients
- Mock authentication providers
- Other service mocks as needed

### 3. Integration Test Utilities

Utilities designed specifically for integration testing:

- Multi-crate test harnesses
- Component wiring helpers
- Test-specific DI container configurations
- Assertion utilities for cross-crate behaviors

### 4. Error Testing Framework

Specialized utilities for testing error scenarios:

- Error injection capabilities
- Error propagation verification
- Context preservation testing
- Error handler testing

## Implementation Approach

### Phase 1: Design and Planning (April 12-15, 2025)

- Define the architecture of the testing infrastructure
- Identify key interfaces that require mock implementations
- Establish patterns for fixture setup and teardown
- Document the approach for different testing scenarios

### Phase 2: Core Infrastructure (April 16-19, 2025)

- Implement the test fixture framework
- Create base mock implementations for core interfaces
- Develop test harness utilities
- Implement configuration mechanisms for tests

### Phase 3: Integration Test Utilities (April 20-23, 2025)

- Implement multi-crate test harnesses
- Create component wiring helpers
- Develop assertion utilities
- Add test-specific DI container configurations

### Phase 4: Documentation and Examples (April 24-26, 2025)

- Document all testing utilities and patterns
- Create example tests for common scenarios
- Develop testing guidelines for contributors
- Update existing tests to use the new infrastructure

## Prototype Implementation

An initial prototype of the Cross-Crate Testing Infrastructure has been created as a starting point for the implementation phase. The prototype includes:

### TestFixture

```rust
/// A test fixture that manages resources and components for tests
#[derive(Clone)]
pub struct TestFixture {
    /// The internal state of the fixture
    state: Arc<Mutex<FixtureState>>,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new() -> TestFixtureBuilder {
        TestFixtureBuilder::new()
    }
    
    /// Register a component with the fixture
    pub fn register<T: Any + Send + Sync>(&self, component: T) -> TestResult<()> {
        // Register components for tests
    }
    
    /// Get a component from the fixture
    pub fn get<T: Any + Send + Sync>(&self) -> TestResult<T> {
        // Retrieve registered components
    }
    
    // Other methods for managing test resources
}
```

### MockRegistry

```rust
/// A registry for mock implementations
#[derive(Clone)]
pub struct MockRegistry {
    /// The internal state of the registry
    state: Arc<Mutex<MockRegistryState>>,
}

impl MockRegistry {
    /// Create a new mock registry
    pub fn new() -> Self {
        // Create a new registry
    }
    
    /// Register a mock implementation for an interface
    pub fn register<Interface, Implementation>(&self, implementation: Implementation) -> TestResult<()>
    where
        Interface: Any + Send + Sync + ?Sized,
        Implementation: Any + Send + Sync,
    {
        // Register mock implementations for interfaces
    }
    
    /// Get a mock implementation for an interface
    pub fn get<Interface, Implementation>(&self) -> TestResult<Implementation>
    where
        Interface: Any + Send + Sync + ?Sized,
        Implementation: Any + Send + Sync + Clone,
    {
        // Retrieve mock implementations
    }
    
    // Other methods for managing mocks
}
```

### TestHarness

```rust
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
        // Create a new test harness
    }
    
    /// Run an async function in the test harness
    pub fn run_async<F, Fut, T>(&self, f: F) -> TestResult<T>
    where
        F: FnOnce(Arc<TestFixture>, Arc<MockRegistry>) -> Fut,
        Fut: Future<Output = TestResult<T>>,
    {
        // Run async tests
    }
    
    /// Run a sync function in the test harness
    pub fn run<F, T>(&self, f: F) -> TestResult<T>
    where
        F: FnOnce(&TestFixture, &MockRegistry) -> TestResult<T>,
    {
        // Run sync tests
    }
    
    // Other methods for test harness management
}
```

## Example Usage

```rust
// Define a simple interface for testing
trait UserService: Send + Sync {
    fn get_user(&self, id: &str) -> Result<User, String>;
}

// Define a mock implementation
#[derive(Clone)]
struct MockUserService {
    users: Vec<User>,
}

impl UserService for MockUserService {
    fn get_user(&self, id: &str) -> Result<User, String> {
        // Mock implementation
    }
}

// Test example
fn test_user_service() -> TestResult<()> {
    let mut harness = TestHarnessBuilder::new()
        .with_runtime()
        .build()?;
    
    harness.run(|fixture, mock_registry| {
        // Register mocks
        let mock_service = MockUserService::new();
        mock_registry.register::<dyn UserService, MockUserService>(mock_service.clone())?;
        
        // Get the user from the mock service
        let user = mock_service.get_user("1")?;
        assert_eq!(user.name, "Test User");
        
        Ok(())
    })
}
```

## Integration with Existing Systems

The Cross-Crate Testing Infrastructure will integrate with:

1. **Component Registry**: For test-specific component registration
2. **Application Framework**: For bootstrapping test environments
3. **Error Handling System**: For testing error propagation
4. **Plugin System**: For testing plugin interactions

## Testing Strategies

### Interface Compliance Testing

Tests that verify implementations comply with interface contracts:

```rust
#[test]
fn test_database_implementation_complies_with_interface() {
    // Use the test framework to verify interface compliance
    let fixture = TestFixture::new()
        .with_postgres_implementation()
        .build();
    
    let implementation = fixture.get_implementation::<dyn DatabaseProvider>();
    
    // Run interface compliance tests
    InterfaceComplianceTester::verify_implementation(implementation);
}
```

### Cross-Crate Integration Testing

Tests that verify interactions between components in different crates:

```rust
#[test]
fn test_auth_and_database_integration() {
    // Set up a test fixture with components from multiple crates
    let fixture = TestFixture::new()
        .with_auth_provider()
        .with_database_provider()
        .build();
    
    // Test the interaction between auth and database components
    let auth_service = fixture.get_service::<AuthService>();
    let db_service = fixture.get_service::<DatabaseService>();
    
    // Verify interactions work as expected
    let user = auth_service.authenticate("test_user", "password").unwrap();
    let user_data = db_service.get_user_data(user.id).unwrap();
    
    assert_eq!(user.id, user_data.id);
}
```

### Error Propagation Testing

Tests that verify errors are properly propagated across crate boundaries:

```rust
#[test]
fn test_error_propagation_across_crates() {
    // Set up a test fixture with error injection
    let fixture = TestFixture::new()
        .with_failing_database_provider()
        .with_auth_provider()
        .build();
    
    // Verify that database errors propagate correctly to the auth service
    let auth_service = fixture.get_service::<AuthService>();
    
    let result = auth_service.get_user_by_id("user1");
    
    // Verify the error is correctly propagated and contains the expected context
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code(), ErrorCode::DatabaseError);
    assert!(error.context().contains("Failed to retrieve user"));
}
```

## Success Criteria

The Cross-Crate Testing Infrastructure will be considered successful when:

1. All major cross-crate interactions have test coverage
2. Error propagation across crate boundaries is thoroughly tested
3. Mocks are available for all key interfaces
4. Documentation and examples make it easy to write new tests
5. CI pipelines include cross-crate testing

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Test performance degradation | Medium | Medium | Optimize fixture setup, use targeted tests |
| Mock implementation drift | High | Medium | Automated verification against real implementations |
| Over-complicated test setup | Medium | High | Focus on developer experience, provide simple helpers |
| Database/external service dependencies | Medium | Medium | Use in-memory implementations for fast tests |

## Next Steps

1. Finalize design details based on the prototype implementation
2. Begin implementation of the test fixture framework on April 12, 2025
3. Create mock implementations for core interfaces
4. Develop integration test utilities
5. Create documentation and examples

*Updated: March 29, 2025* 