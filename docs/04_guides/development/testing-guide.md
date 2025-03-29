---
title: "Testing Guide for Navius Development"
description: "Comprehensive guide for writing and running tests in Navius applications"
category: "Guides"
tags: ["development", "testing", "quality assurance", "unit tests", "integration tests", "e2e tests"]
last_updated: "April 7, 2025"
version: "1.0"
---

# Testing Guide for Navius Development

This guide provides comprehensive instructions for testing Navius applications. Quality testing ensures reliability, improves maintainability, and accelerates development by catching issues early.

## Table of Contents

- [Testing Philosophy](#testing-philosophy)
- [Test Types and Structure](#test-types-and-structure)
- [Writing Effective Tests](#writing-effective-tests)
- [Test Organization](#test-organization)
- [Test Frameworks and Tools](#test-frameworks-and-tools)
- [Running Tests](#running-tests)
- [Test Coverage](#test-coverage)
- [Testing Best Practices](#testing-best-practices)
- [Mocking and Test Doubles](#mocking-and-test-doubles)
- [Continuous Integration](#continuous-integration)
- [Debugging Tests](#debugging-tests)

## Testing Philosophy

Navius follows these testing principles:

1. **Test Early, Test Often** - Tests should be written alongside code development
2. **Test Isolation** - Tests should be independent and not affect each other
3. **Test Readability** - Tests serve as documentation and should be clear and understandable
4. **Speed Matters** - The test suite should run quickly to enable frequent testing
5. **Risk-Based Testing** - Focus more testing efforts on critical and complex components

## Test Types and Structure

### Unit Tests

Unit tests verify individual components in isolation. In Navius, unit tests are typically:

- Located in the same file as the code they're testing, in a `tests` module
- Focused on a single function or method
- Fast to execute
- Don't require external resources

Example unit test:

```rust
// In src/utils/string_utils.rs
pub fn capitalize(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_empty_string() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_capitalize_single_letter() {
        assert_eq!(capitalize("a"), "A");
    }

    #[test]
    fn test_capitalize_word() {
        assert_eq!(capitalize("hello"), "Hello");
    }
    
    #[test]
    fn test_capitalize_already_capitalized() {
        assert_eq!(capitalize("Hello"), "Hello");
    }
}
```

### Integration Tests

Integration tests verify that different components work together correctly. In Navius:

- Located in the `tests/` directory at the project root
- Test interactions between multiple components
- May use test databases or other isolated resources
- Focus on component interfaces and interactions

Example integration test:

```rust
// In tests/auth_integration_test.rs
use navius::{
    auth::{AuthService, User},
    database::{self, Database},
};
use uuid::Uuid;

#[tokio::test]
async fn test_user_authentication_flow() {
    // Setup
    let db = database::get_test_database().await.unwrap();
    let auth_service = AuthService::new(db.clone());
    
    // Create test user
    let username = format!("test_user_{}", Uuid::new_v4());
    let password = "secureP@ssw0rd";
    
    // Register
    let user = auth_service.register(&username, password).await.unwrap();
    assert_eq!(user.username, username);
    
    // Login
    let login_result = auth_service.login(&username, password).await.unwrap();
    assert!(login_result.token.len() > 10);
    
    // Verify
    let user_id = user.id;
    let verified = auth_service.verify_token(&login_result.token).await.unwrap();
    assert_eq!(verified.user_id, user_id);
    
    // Cleanup
    db.delete_user(user_id).await.unwrap();
}
```

### End-to-End Tests

E2E tests verify complete user flows through the system. In Navius:

- Located in the `tests/e2e/` directory
- Test complete user flows and scenarios
- Often use browser automation tools like Selenium or Playwright
- Slower but provide high confidence in the system's correctness

Example E2E test (using Playwright for web UI testing):

```typescript
// In tests/e2e/user_registration.spec.ts
import { test, expect } from '@playwright/test';

test.describe('User Registration Flow', () => {
  test('should allow a new user to register and login', async ({ page }) => {
    // Generate unique username
    const username = `test_user_${Date.now()}`;
    const password = 'SecureP@ss123';
    
    // Visit registration page
    await page.goto('/register');
    
    // Fill and submit registration form
    await page.fill('[data-testid="username-input"]', username);
    await page.fill('[data-testid="password-input"]', password);
    await page.fill('[data-testid="confirm-password-input"]', password);
    await page.click('[data-testid="register-button"]');
    
    // Verify successful registration
    await expect(page).toHaveURL('/login');
    
    // Login with new credentials
    await page.fill('[data-testid="username-input"]', username);
    await page.fill('[data-testid="password-input"]', password);
    await page.click('[data-testid="login-button"]');
    
    // Verify successful login
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('[data-testid="user-greeting"]')).toContainText(username);
  });
});
```

### API Tests

API tests verify API endpoints. In Navius:

- Located in the `tests/api/` directory
- Test API request/response cycles
- Validate response status, headers, and body
- Can use libraries like reqwest or testing frameworks like Postman

Example API test:

```rust
// In tests/api/user_api_test.rs
use navius::setup_test_server;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};

#[tokio::test]
async fn test_user_creation_api() {
    // Start test server
    let server = setup_test_server().await;
    let client = Client::new();
    let base_url = format!("http://localhost:{}", server.port());
    
    // Create user request
    let response = client
        .post(&format!("{}/api/users", base_url))
        .json(&json!({
            "username": "api_test_user",
            "password": "P@ssw0rd123",
            "email": "api_test@example.com"
        }))
        .send()
        .await
        .unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let user: Value = response.json().await.unwrap();
    assert_eq!(user["username"], "api_test_user");
    assert_eq!(user["email"], "api_test@example.com");
    assert!(user.get("password").is_none()); // Password should not be returned
    
    // Verify user was created by fetching it
    let get_response = client
        .get(&format!("{}/api/users/{}", base_url, user["id"]))
        .send()
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), StatusCode::OK);
    
    // Cleanup
    let delete_response = client
        .delete(&format!("{}/api/users/{}", base_url, user["id"]))
        .send()
        .await
        .unwrap();
    
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}
```

## Writing Effective Tests

### Test Structure

Follow the AAA (Arrange-Act-Assert) pattern for clear test structure:

```rust
#[test]
fn test_user_validation() {
    // Arrange
    let user_input = UserInput {
        username: "user1",
        email: "invalid-email",
        password: "short",
    };
    let validator = UserValidator::new();
    
    // Act
    let validation_result = validator.validate(&user_input);
    
    // Assert
    assert!(!validation_result.is_valid);
    assert_eq!(validation_result.errors.len(), 2);
    assert!(validation_result.errors.contains(&ValidationError::InvalidEmail));
    assert!(validation_result.errors.contains(&ValidationError::PasswordTooShort));
}
```

### Descriptive Test Names

Use descriptive test names that explain what is being tested and the expected outcome:

```rust
// Not descriptive
#[test]
fn test_user() { /* ... */ }

// More descriptive
#[test]
fn test_user_with_invalid_email_should_fail_validation() { /* ... */ }
```

### Testing Edge Cases

Include tests for edge cases and boundary conditions:

```rust
#[test]
fn test_pagination_with_zero_items() { /* ... */ }

#[test]
fn test_pagination_with_exactly_one_page() { /* ... */ }

#[test]
fn test_pagination_with_partial_last_page() { /* ... */ }

#[test]
fn test_pagination_with_max_page_size() { /* ... */ }
```

## Test Organization

### Directory Structure

Navius follows this test organization:

```
navius/
├── src/
│   ├── module1/
│   │   ├── file1.rs (with unit tests)
│   │   └── file2.rs (with unit tests)
│   └── module2/
│       └── file3.rs (with unit tests)
├── tests/
│   ├── integration/
│   │   ├── module1_test.rs
│   │   └── module2_test.rs
│   ├── api/
│   │   ├── endpoints_test.rs
│   │   └── middleware_test.rs
│   ├── e2e/
│   │   └── user_flows_test.rs
│   └── common/
│       └── test_helpers.rs
```

### Test Tagging

Use attributes to categorize and run specific test groups:

```rust
#[test]
#[ignore = "slow test, run only in CI"]
fn test_intensive_operation() { /* ... */ }

#[test]
#[cfg(feature = "extended-tests")]
fn test_extended_feature() { /* ... */ }
```

## Test Frameworks and Tools

### Core Testing Frameworks

- **Rust's built-in test framework** - For unit and integration tests
- **tokio::test** - For async testing
- **Criterion** - For benchmarking
- **Playwright/Selenium** - For E2E tests (frontend)
- **reqwest** - For API testing

### Helper Libraries

- **pretty_assertions** - For improved assertion output
- **mock_it** - For mocking in Rust
- **rstest** - For parameterized tests
- **test-case** - For table-driven tests
- **fake** - For generating test data

## Running Tests

### Basic Test Execution

```bash
# Run all tests
cargo test

# Run tests in a specific file
cargo test --test auth_integration_test

# Run tests with a specific name pattern
cargo test user_validation

# Run ignored tests
cargo test -- --ignored

# Run a specific test
cargo test test_user_with_invalid_email_should_fail_validation
```

### Test Configuration

Configure test behavior using environment variables or the `.env.test` file:

```
# .env.test
TEST_DATABASE_URL=postgres://postgres:password@localhost:5432/navius_test
TEST_REDIS_URL=redis://localhost:6379/1
TEST_LOG_LEVEL=debug
```

Load these in your test setup:

```rust
use dotenv::dotenv;
use std::env;

fn setup() {
    dotenv::from_filename(".env.test").ok();
    let db_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    // Use db_url for test database connection
}
```

## Test Coverage

### Measuring Coverage

Navius uses [grcov](https://github.com/mozilla/grcov) for test coverage:

```bash
# Install grcov
cargo install grcov

# Generate coverage report
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./coverage/
```

### Coverage Targets

- Minimum coverage targets:
  - 80% line coverage for business logic
  - 70% branch coverage for business logic
  - 60% line coverage for infrastructure code

## Testing Best Practices

### Do's:

- ✅ Write tests before or alongside code (TDD/BDD when possible)
- ✅ Keep tests independent and isolated
- ✅ Use meaningful test data
- ✅ Test failure cases, not just success paths
- ✅ Run tests frequently during development
- ✅ Test public interfaces rather than implementation details
- ✅ Clean up test resources (connections, files, etc.)

### Don'ts:

- ❌ Don't skip testing error conditions
- ❌ Don't use random data without controlling the seed
- ❌ Don't write tests that depend on execution order
- ❌ Don't test trivial code (e.g., getters/setters)
- ❌ Don't write overly complex tests
- ❌ Don't include external services in unit tests

## Mocking and Test Doubles

### Types of Test Doubles

1. **Stubs** - Return predefined responses
2. **Mocks** - Verify expected interactions
3. **Fakes** - Working implementations for testing only
4. **Spies** - Record calls for later verification

### Mocking in Rust

Example using the `mockall` crate:

```rust
use mockall::{automock, predicate::*};

#[automock]
trait Database {
    fn get_user(&self, id: u64) -> Option<User>;
    fn save_user(&self, user: &User) -> Result<(), DbError>;
}

#[test]
fn test_user_service_with_mock_db() {
    let mut mock_db = MockDatabase::new();
    
    // Setup expectations
    mock_db.expect_get_user()
        .with(predicate::eq(42))
        .times(1)
        .returning(|_| Some(User { id: 42, name: "Test User".to_string() }));
    
    // Create service with mock
    let user_service = UserService::new(Box::new(mock_db));
    
    // Test the service
    let user = user_service.get_user(42).unwrap();
    assert_eq!(user.name, "Test User");
}
```

### Creating Test Fakes

For complex dependencies, create fake implementations:

```rust
// A fake in-memory database for testing
struct InMemoryDatabase {
    users: std::sync::Mutex<HashMap<u64, User>>,
}

impl InMemoryDatabase {
    fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl Database for InMemoryDatabase {
    fn get_user(&self, id: u64) -> Option<User> {
        self.users.lock().unwrap().get(&id).cloned()
    }
    
    fn save_user(&self, user: &User) -> Result<(), DbError> {
        self.users.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }
}
```

## Continuous Integration

### CI Test Configuration

Navius uses GitHub Actions for CI testing:

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_USER: postgres
          POSTGRES_DB: navius_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:6
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: rustfmt, clippy
      
      - name: Cache dependencies
        uses: actions/cache@v2
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --all-features
        env:
          TEST_DATABASE_URL: postgres://postgres:postgres@localhost:5432/navius_test
          TEST_REDIS_URL: redis://localhost:6379/1
      
      - name: Generate coverage
        run: |
          cargo install grcov
          CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
          grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing -o ./lcov.info
      
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v1
        with:
          file: ./lcov.info
          fail_ci_if_error: false
```

## Debugging Tests

### Tips for Debugging Tests

1. **Use test-specific logging**:
   ```rust
   #[test]
   fn test_complex_operation() {
       let _ = env_logger::builder().is_test(true).try_init();
       debug!("Starting test with parameters: {:?}", test_params);
       
       // Test code...
   }
   ```

2. **Run single tests with verbose output**:
   ```bash
   RUST_LOG=debug cargo test test_name -- --nocapture
   ```

3. **Use the debugger**:
   Configure your IDE to debug tests, set breakpoints, and step through code.

4. **Add more detailed assertions**:
   ```rust
   // Instead of
   assert_eq!(result, expected);
   
   // Use more descriptive assertions
   assert_eq!(
       result, 
       expected,
       "Result {:?} doesn't match expected {:?} when processing input {:?}",
       result, expected, input
   );
   ```

### Common Test Failures

- **Failing Async Tests**: Ensure your runtime is properly set up and test futures are awaited
- **Flaky Tests**: Look for race conditions or external dependencies
- **Timeout Issues**: Check for blocking operations in async contexts
- **Resource Leaks**: Ensure proper cleanup after tests

## Related Resources

- [Official Rust Test Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Navius Testing Templates](../contributing/test-implementation-template.md)
- [Code Coverage Reports](https://coverage.navius.dev)
- [Testing Guidelines](../contributing/testing-guidelines.md)
- [Debugging Guide](./debugging-guide.md)
