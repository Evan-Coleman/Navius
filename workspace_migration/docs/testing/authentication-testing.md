# Authentication Testing Guide

This guide covers best practices for testing authentication in applications using the Navius Test Framework. It complements the code example in `workspace_migration/examples/crates/navius-test/examples/authentication_testing.rs`.

## Overview

Authentication is a critical aspect of application security that requires thorough testing. The Navius Test Framework provides tools and utilities to simplify authentication testing across different components of your application.

## Key Authentication Testing Scenarios

Effective authentication testing should cover these key scenarios:

1. **Basic Authentication Flow**
   - Credential validation
   - Token generation and validation
   - Session management

2. **Multi-Factor Authentication (MFA)**
   - MFA challenge generation
   - MFA code validation
   - Fallback mechanisms

3. **Token Lifecycle**
   - Token creation
   - Token validation
   - Token refresh
   - Token invalidation/revocation

4. **Authorization**
   - Role-based access control
   - Permission checking
   - Scope validation

5. **Error Handling**
   - Invalid credentials
   - Expired tokens
   - Access denied scenarios
   - Rate limiting

## Using Mock Authentication Components

The Navius Test Framework provides mock authentication components that simplify testing:

### MockAuthClient

The `MockAuthClient` allows you to simulate authentication operations:

```rust
use navius_test::mocks::auth::MockAuthClient;

let auth_client = MockAuthClient::new();

// Set up expectations
auth_client.expect_validate_credentials("username", "password")
    .returns(Ok("user123".to_string()));

auth_client.expect_generate_token("user123", &["user:read".to_string()])
    .returns(Ok("jwt.token.here".to_string()));
```

### MockUserStore

The `MockUserStore` allows you to simulate user data storage:

```rust
use navius_test::mocks::auth::{MockUserStore, UserInfo, Role};

let user_store = MockUserStore::new();

// Set up expectations
user_store.expect_get_user("user123")
    .returns(Ok(UserInfo {
        id: "user123".to_string(),
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        roles: vec![Role::User],
        mfa_enabled: false,
        mfa_methods: vec![],
    }));
```

## Testing Authentication Flows

### Basic Authentication Flow

Test the complete authentication flow from credential validation to token generation:

```rust
#[tokio::test]
async fn test_basic_authentication() -> TestResult<()> {
    // Create test fixture
    let fixture = TestFixture::new();
    
    // Create and configure mock components
    let auth_client = MockAuthClient::new();
    let user_store = MockUserStore::new();
    
    // Set up expectations
    // ...
    
    // Create service and test authentication
    let service = create_authentication_service(auth_client, user_store);
    let result = service.authenticate(create_auth_request()).await?;
    
    // Verify the result
    assert_token_valid(result)?;
    
    Ok(())
}
```

### Multi-Factor Authentication

Test MFA flows, including initial authentication and MFA completion:

```rust
#[tokio::test]
async fn test_mfa_authentication() -> TestResult<()> {
    // Create test fixture and mock components
    // ...
    
    // Test initial authentication (should return MFA challenge)
    let initial_result = service.authenticate(request).await?;
    let temp_token = assert_requires_mfa(initial_result)?;
    
    // Test MFA completion
    let mfa_result = service.complete_mfa(temp_token, "123456").await?;
    assert_authentication_successful(mfa_result)?;
    
    Ok(())
}
```

## Testing Authorization

Test that authorization correctly enforces access control rules:

```rust
#[tokio::test]
async fn test_authorization() -> TestResult<()> {
    // Create test fixture and mock components
    // ...
    
    // Test admin access (should succeed)
    let admin_request = create_access_request("admin.token", "reports", "read");
    let admin_result = service.authorize(admin_request).await?;
    assert_true(admin_result, "Admin should have access")?;
    
    // Test user access (should fail)
    let user_request = create_access_request("user.token", "reports", "read");
    let user_result = service.authorize(user_request).await?;
    assert_false(user_result, "User should not have access")?;
    
    Ok(())
}
```

## Testing Token Lifecycle

Test token validation, expiration, and refresh:

```rust
#[tokio::test]
async fn test_token_lifecycle() -> TestResult<()> {
    // Create test fixture and mock components
    // ...
    
    // Test valid token
    let valid_result = service.validate_token("valid.token").await;
    assert_ok(&valid_result, "Valid token should be accepted")?;
    
    // Test expired token
    let expired_result = service.validate_token("expired.token").await;
    assert_err(expired_result, "Expired token should be rejected")?;
    
    // Test token refresh
    let refresh_result = service.refresh_token("expiring.token").await?;
    assert_ne(refresh_result.value, "expiring.token", "Should issue a new token")?;
    
    Ok(())
}
```

## Testing Error Handling

Test various error scenarios to ensure proper handling:

```rust
#[tokio::test]
async fn test_authentication_errors() -> TestResult<()> {
    // Create test fixture and mock components
    // ...
    
    // Test invalid credentials
    auth_client.expect_validate_credentials("bad_user", "wrong_pass")
        .returns(Err(AuthError::InvalidCredentials));
    
    let result = service.authenticate(bad_request).await;
    assert_err(result, "Invalid credentials should be rejected")?;
    assert_matches!(result.unwrap_err(), AuthError::InvalidCredentials);
    
    Ok(())
}
```

## Best Practices

### 1. Test the Complete Flow

Test the entire authentication flow from login to token validation to logout, including any intermediate steps like MFA.

### 2. Use Deterministic Mocks

Configure mocks to return predictable, controlled responses to ensure your tests are deterministic and reliable.

### 3. Test Edge Cases

Test various edge cases such as:
- Expired tokens
- Invalid credentials
- Missing MFA codes
- Invalid permissions
- Token refresh near expiry

### 4. Isolate Tests

Ensure that authentication tests are isolated from each other and don't depend on shared state.

### 5. Verify Token Properties

When testing token generation, verify:
- The token contains the expected user ID
- The token has the correct scopes/permissions
- The token has an appropriate expiration time

### 6. Test Security Boundaries

Explicitly test that users cannot access resources they're not authorized for.

### 7. Use Helper Functions

Create helper functions for common assertions to make tests more readable and maintainable:

```rust
fn assert_token_valid(result: AuthResult) -> TestResult<()> {
    match result {
        AuthResult::Success(token) => {
            assert_eq(token.user_id, expected_id, "Should have the correct user ID")?;
            // More assertions...
            Ok(())
        },
        _ => Err("Expected successful authentication".into()),
    }
}
```

## Common Pitfalls

1. **Time-Dependent Tests**: Be careful with tests that depend on token expiration times. Consider making time-based checks relative rather than absolute.

2. **Testing Too Broadly**: Focus each test on a specific aspect of authentication rather than trying to test everything at once.

3. **Insufficient Error Testing**: Don't just test the happy path; be thorough in testing error conditions.

4. **Hardcoded Credentials**: Avoid hardcoding real credentials in tests. Use mock authentication providers instead.

5. **Missing Role/Permission Tests**: Ensure you test different role combinations and permission levels.

## Integration with Real Authentication Providers

For end-to-end tests, you may need to test with real authentication providers. The Navius Test Framework supports this through integration fixtures:

```rust
#[tokio::test]
async fn test_real_auth_integration() -> TestResult<()> {
    // Create integration test context
    let ctx = IntegrationContext::new()?;
    
    // Set up test auth provider
    let auth_provider = ctx.create_test_auth_provider()?;
    
    // Create a test user
    let test_user = auth_provider.create_test_user("test@example.com", "TestPassword123!")?;
    
    // Test authentication with real provider
    // ...
    
    // Clean up
    auth_provider.delete_test_user(test_user.id)?;
    
    Ok(())
}
```

## Conclusion

Effective authentication testing is crucial for application security. By using the Navius Test Framework's authentication testing utilities, you can thoroughly test authentication flows, ensuring that your application correctly validates user identity and enforces access control.

For a complete code example, refer to `workspace_migration/examples/crates/navius-test/examples/authentication_testing.rs`. 