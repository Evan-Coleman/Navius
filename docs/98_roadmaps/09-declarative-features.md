---
title: "Declarative Programming Features Roadmap"
description: "Documentation about Declarative Programming Features Roadmap"
category: roadmap
tags:
  - api
  - documentation
  - integration
  - performance
  - security
  - testing
last_updated: March 27, 2025
version: 1.0
---
# Declarative Programming Features Roadmap

## Overview
A lightweight approach to implementing essential declarative features in Navius, focusing on security, developer experience, and performance. We aim to provide a set of Rust-idiomatic procedural macros and utilities that enhance productivity while maintaining Rust's performance and safety guarantees. Our focus is on practical, commonly-used patterns that reduce boilerplate without sacrificing clarity or type safety.

## Current State
- Manual implementation required for cross-cutting concerns
- Basic error handling patterns established
- Initial procedural macro infrastructure in place
- Prototype validation macros implemented
- Basic integration with Axum extractors
- Test coverage for existing macros at 95%

## Target State
A comprehensive set of declarative features that:
- Improve security through consistent validation and error handling
- Enhance developer experience with reduced boilerplate
- Maintain Rust's performance characteristics
- Seamlessly integrate with Axum's middleware and extractor system
- Support compile-time validation and type checking
- Provide clear error messages during compilation
- Enable easy testing and mocking
- Include comprehensive documentation and examples

## Implementation Progress Tracking

### Phase 1: Essential Validation and Error Handling
1. **Request Validation**
   - [x] Set up procedural macro infrastructure
   - [x] Create basic `#[validate_request]` attribute
   - [ ] Implement field-level validation rules
     - [ ] String length and pattern validation
     - [ ] Numeric range validation
     - [ ] Custom validation functions
   - [ ] Add support for nested validation
   - [ ] Integrate with validator crate
   - [ ] Implement custom validation error types
   - [ ] Add validation error formatting
   - [ ] Create validation middleware
   
   *Updated at: March 24, 2025 - Basic infrastructure complete, working on validation rules*

2. **Error Handling**
   - [x] Create `#[api_handler]` macro
   - [x] Implement basic error conversion
   - [ ] Add error categorization
     - [ ] HTTP status code mapping
     - [ ] Error code generation
     - [ ] Error message templating
   - [ ] Implement error context propagation
   - [ ] Add request tracing integration
   - [ ] Create error reporting utilities
   
   *Updated at: March 24, 2025 - Core functionality working, expanding features*

3. **Authorization Checks**
   - [ ] Implement `#[require_permission]` macro
     - [ ] Permission parsing and validation
     - [ ] Role hierarchy support
     - [ ] Custom authorization rules
   - [ ] Add attribute-based access control
   - [ ] Implement permission composition
   - [ ] Create audit logging system
   - [ ] Add authorization testing utilities
   
   *Updated at: Not started*

### Phase 2: Performance and Developer Experience 
1. **Rate Limiting**
   - [ ] Create `#[rate_limit]` attribute
     - [ ] Request counting implementation
     - [ ] Time window management
     - [ ] Rate limit key extraction
   - [ ] Add distributed rate limiting
   - [ ] Implement adaptive rate limiting
   - [ ] Create rate limit monitoring
   
   *Updated at: Not started*

2. **Logging and Instrumentation**
   - [ ] Implement `#[traced]` attribute
     - [ ] Automatic span creation
     - [ ] Context propagation
     - [ ] Log level configuration
   - [ ] Add performance metrics
     - [ ] Timing measurements
     - [ ] Counter increments
     - [ ] Histogram recording
   - [ ] Create structured logging helpers
   - [ ] Add log sampling configuration
   
   *Updated at: Not started*

### Phase 3: Axum Integration
1. **Routing Integration**
   - [ ] Create route definition macros
     - [ ] Path parameter extraction
     - [ ] Query parameter handling
     - [ ] Response type inference
   - [ ] Add OpenAPI generation
   - [ ] Implement middleware composition
   - [ ] Create response transformers
   
   *Updated at: Not started*

2. **Middleware Integration**
   - [ ] Create middleware factory macros
     - [ ] State injection
     - [ ] Error handling
     - [ ] Request/response transformation
   - [ ] Implement ordering system
   - [ ] Add conditional middleware
   - [ ] Create middleware testing utilities
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 15% complete
- **Last Updated**: March 24, 2025
- **Next Milestone**: Complete Request Validation Phase
- **Current Focus**: Field-level validation rules

## Success Criteria
- Security requirements consistently enforced
- Developer productivity improved with 50% less boilerplate
- Performance overhead under 1ms per request
- Seamless Axum integration
- Comprehensive compile-time checks
- Clear error messages
- 100% test coverage for macros
- Detailed documentation with examples

## Implementation Notes

### Example Implementation

```rust
use navius_macros::*;
use validator::Validate;
use serde::Deserialize;

// Enhanced validation with custom rules
#[derive(Deserialize, Validate)]
#[validate(schema(function = "validate_user_data"))]
struct CreateUserRequest {
    #[validate(length(min = 1, max = 100), custom = "validate_name")]
    name: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(range(min = 0, max = 150))]
    age: u8,
    
    #[validate(nested)]
    address: AddressData,
}

// Example of a fully decorated endpoint
#[api_handler]
#[validate_request]
#[rate_limit(requests = 5, period = "1m", key = "ip")]
#[require_permission("users.write")]
#[traced(name = "create_user", fields(user_email = "payload.email"))]
async fn create_user(
    State(db): State<Arc<DbService>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    #[validate] Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Business logic (validation and auth already handled by macros)
    let user = db.create_user(&payload).await.context("Failed to create user")?;
    
    // Automatic response handling
    Ok(Json(UserCreatedResponse::from(user)))
}

// Example of middleware composition
#[middleware_stack]
#[trace_requests]
#[rate_limit]
#[auth_required]
async fn protected_api(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Middleware logic here
    next.run(request).await
}

// Example of a service with declarative error handling
#[derive(Service)]
#[error_handler(strategy = "retry", max_attempts = 3)]
struct UserService {
    #[inject]
    db: Arc<DbService>,
    
    #[inject]
    cache: Arc<CacheService>,
}

#[async_trait]
impl UserServiceTrait for UserService {
    #[traced(level = "debug", metrics = true)]
    async fn get_user(&self, id: Uuid) -> Result<User, ServiceError> {
        // Service logic with automatic tracing and metrics
        if let Some(user) = self.cache.get(id).await? {
            return Ok(user);
        }
        
        let user = self.db.get_user(id).await?;
        self.cache.set(id, &user).await?;
        Ok(user)
    }
}

// Example of compile-time validation
const fn validate_config<T: Config>() {
    assert!(T::MAX_CONNECTIONS > 0, "MAX_CONNECTIONS must be positive");
    assert!(T::TIMEOUT_MS > 1000, "TIMEOUT_MS must be at least 1 second");
}
```

### Testing Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use navius_test::*;

    #[tokio::test]
    async fn test_validated_endpoint() {
        // Arrange
        let app = test_app()
            .with_auth()
            .with_rate_limit()
            .build()
            .await;
            
        // Act
        let response = app
            .post("/users")
            .json(&invalid_user_data())
            .send()
            .await;
            
        // Assert
        assert_validation_error!(
            response,
            includes_field("email"),
            has_code("INVALID_FORMAT")
        );
    }
    
    #[tokio::test]
    async fn test_rate_limited_endpoint() {
        // Arrange
        let app = test_app().build().await;
        
        // Act & Assert
        for _ in 0..5 {
            let response = app.get("/api").send().await;
            assert_ok!(response);
        }
        
        let response = app.get("/api").send().await;
        assert_rate_limited!(response);
    }
}
```

## References
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Rust Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Validator Crate](https://docs.rs/validator/latest/validator/)
- [Tower Service Traits](https://docs.rs/tower/latest/tower/trait.Service.html)
- [Tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [syn Crate](https://docs.rs/syn/latest/syn/)
- [quote Crate](https://docs.rs/quote/latest/quote/)
- [proc-macro2 Crate](https://docs.rs/proc-macro2/latest/proc_macro2/) 

## Related Documents
- [Project Structure Roadmap](../completed/11_project_structure_future_improvements.md) - Future improvements
- [Documentation Overhaul](../12_document_overhaul.md) - Documentation plans

