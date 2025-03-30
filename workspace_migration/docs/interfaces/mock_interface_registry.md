# Mock Interface Registry

## Overview

The Mock Interface Registry is a core component of the Navius Test Framework, providing a centralized system for registering, configuring, and retrieving mock implementations of interfaces used throughout the Navius application. This system enables developers to create isolated test environments where external dependencies can be controlled and verified.

## Key Components

### 1. Registry Core

The core of the mock interface registry is built around a type-safe registration system:

```rust
pub struct MockRegistry {
    mocks: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    expectations: RwLock<HashMap<String, Vec<Expectation>>>,
}
```

Key features:
- Thread-safe access to registered mocks using `RwLock`
- Type identification via Rust's `TypeId` system
- Dynamic storage using Rust's `Any` trait
- Support for concurrent test execution

### 2. Expectation Management

The registry implements a powerful expectation system that allows tests to define expected method calls and their results:

```rust
pub struct Expectation {
    method: String,
    args: Option<Box<dyn Any + Send + Sync>>,
    result: Option<Box<dyn Any + Send + Sync>>,
    times: ExpectationTimes,
    called: AtomicUsize,
}

pub enum ExpectationTimes {
    Any,
    Exactly(usize),
    AtLeast(usize),
    AtMost(usize),
    Between(usize, usize),
}
```

This system allows for flexible verification of mock interactions:
- Method names are tracked with their arguments
- Results can be predefined for specific method calls
- Call count expectations can be defined with various constraints
- Atomic counters ensure thread safety during verification

### 3. API Overview

The registry provides a clean API for working with mocks:

```rust
impl MockRegistry {
    pub fn new() -> Self { /* ... */ }
    
    pub fn register<T: 'static + Send + Sync>(&self, mock: T) -> TestResult<()> { /* ... */ }
    
    pub fn get<T: 'static + Clone + Send + Sync>(&self) -> TestResult<T> { /* ... */ }
    
    pub fn expect(&self, interface: &str, method: &str) -> ExpectationBuilder { /* ... */ }
    
    pub fn verify(&self) -> TestResult<()> { /* ... */ }
    
    pub fn reset(&self) -> TestResult<()> { /* ... */ }
}
```

## Mock Implementation Pattern

All mock implementations follow a consistent pattern:

```rust
pub struct MockDatabaseClient {
    registry: Arc<MockRegistry>,
    name: String,
}

impl MockDatabaseClient {
    pub fn new(registry: Arc<MockRegistry>) -> Self {
        let instance = Self {
            registry,
            name: "DatabaseClient".to_string(),
        };
        // Register instance in registry
        instance
    }

    // Helper method to record method calls
    fn record_call(&self, method: &str, args: Option<Box<dyn Any + Send + Sync>>) -> TestResult<()> {
        /* ... */
    }

    // Helper to get expected result for a method call
    fn get_result<T: 'static + Clone>(&self, method: &str, args: Option<Box<dyn Any + Send + Sync>>) -> TestResult<T> {
        /* ... */
    }
}

// Implement actual interface
impl DatabaseClient for MockDatabaseClient {
    fn execute(&self, query: &str, params: &[Value]) -> Result<usize, DbError> {
        let args = Some(Box::new((query.to_string(), params.to_vec())));
        self.record_call("execute", args.clone())?;
        let result: Result<usize, DbError> = self.get_result("execute", args)?;
        result
    }
    
    // Other methods similarly implemented
}
```

## Implemented Mock Interfaces

### 1. Database Interface Mocks

The `MockDatabaseClient` provides a comprehensive mock implementation of the Navius database interface:

```rust
pub trait DatabaseClient {
    fn execute(&self, query: &str, params: &[Value]) -> Result<usize, DbError>;
    fn query(&self, query: &str, params: &[Value]) -> Result<Vec<Row>, DbError>;
    fn transaction(&self) -> Result<Transaction, DbError>;
    // Additional methods...
}
```

The mock implementation allows tests to:
- Verify SQL queries are executed with correct parameters
- Return predefined results for specific queries
- Simulate database errors and edge cases
- Track transaction operations

### 2. Filesystem Interface Mocks

The `MockFileSystem` provides a virtual file system for testing file operations:

```rust
pub trait FileSystem {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, io::Error>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), io::Error>;
    fn create_dir(&self, path: &Path) -> Result<(), io::Error>;
    fn exists(&self, path: &Path) -> bool;
    // Additional methods...
}
```

The mock implementation allows tests to:
- Simulate file and directory structures without real filesystem access
- Verify file operations occur with correct paths and content
- Inject I/O errors to test error handling
- Track file access patterns

## Integration with Test Fixture

The Mock Interface Registry integrates seamlessly with the TestFixture system:

```rust
let fixture = TestFixture::new();
let registry = MockRegistry::new();

// Register the registry with the fixture
fixture.register_component(registry.clone())?;

// Create and register a mock database
let db_mock = MockDatabaseClient::new(registry.clone());
fixture.register_component::<dyn DatabaseClient>(db_mock.clone())?;

// Configure expectations
registry.expect("DatabaseClient", "query")
    .with(("SELECT * FROM users", vec![]))
    .times(1)
    .returns(Ok(vec![Row::new(vec![("id", Value::Integer(1)), ("name", Value::Text("Test User"))])]));

// Run test with the fixture
let result = test_with_fixture(&fixture);

// Verify all expectations were met
registry.verify()?;
```

## Usage Examples

### Basic Mock Usage

```rust
#[test]
fn test_user_repository() -> TestResult<()> {
    // Setup
    let fixture = TestFixture::new();
    let registry = MockRegistry::new();
    fixture.register_component(registry.clone())?;
    
    let mock_db = MockDatabaseClient::new(registry.clone());
    fixture.register_component::<dyn DatabaseClient>(mock_db.clone())?;
    
    // Configure expectations
    registry.expect("DatabaseClient", "query")
        .with(("SELECT * FROM users WHERE id = ?", vec![Value::Integer(1)]))
        .returns(Ok(vec![Row::new(vec![
            ("id", Value::Integer(1)),
            ("name", Value::Text("John Doe"))
        ])]));
    
    // Create the system under test using the fixture
    let user_repo = UserRepository::new(fixture.get_component::<dyn DatabaseClient>()?);
    
    // Exercise
    let user = user_repo.get_by_id(1)?;
    
    // Verify
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "John Doe");
    registry.verify()?;
    
    Ok(())
}
```

### Testing Error Handling

```rust
#[test]
fn test_user_repository_error_handling() -> TestResult<()> {
    // Setup
    let fixture = TestFixture::new();
    let registry = MockRegistry::new();
    fixture.register_component(registry.clone())?;
    
    let mock_db = MockDatabaseClient::new(registry.clone());
    fixture.register_component::<dyn DatabaseClient>(mock_db.clone())?;
    
    // Configure expectations - simulate a database error
    registry.expect("DatabaseClient", "query")
        .with(("SELECT * FROM users WHERE id = ?", vec![Value::Integer(1)]))
        .returns(Err(DbError::ConnectionFailed("Test error".into())));
    
    // Create the system under test
    let user_repo = UserRepository::new(fixture.get_component::<dyn DatabaseClient>()?);
    
    // Exercise - should propagate the error
    let result = user_repo.get_by_id(1);
    
    // Verify
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_matches!(err, UserError::DatabaseError(_));
    
    registry.verify()?;
    
    Ok(())
}
```

## Next Steps

With the Mock Interface Registry complete, the next phase of development will focus on:

1. **Integration Test Utilities**
   - Building on the mock interface registry to create higher-level test abstractions
   - Developing utilities for cross-crate integration testing
   - Creating test runners for different test scenarios

2. **Documentation and Examples**
   - Creating comprehensive guides for using the mock interfaces
   - Developing example tests for common testing patterns
   - Documenting best practices for mocking in the Navius ecosystem

3. **Additional Mock Interfaces**
   - HTTP client interface mocks
   - Configuration service mocks
   - Authentication service mocks
   - Cache service mocks
   - Logger interface mocks 