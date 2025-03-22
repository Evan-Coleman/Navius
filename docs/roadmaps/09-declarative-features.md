# Declarative Programming Features Roadmap

## Overview
A lightweight approach to implementing essential declarative features in Navius, focusing on security, developer experience, and performance. Rather than attempting to recreate Spring Boot's extensive annotation system, we'll implement a targeted set of Rust-idiomatic procedural macros and utilities that provide the most value with minimal complexity.

## Current State
Currently, our application requires manual implementation of cross-cutting concerns like validation, error handling, and logging, leading to repetitive code and potential inconsistencies.

## Target State
A small but powerful set of declarative features that:
- Improve security by ensuring consistent validation and error handling
- Enhance developer experience with less boilerplate
- Maintain Rust's performance characteristics
- Work seamlessly with Axum's middleware and extractor system
- Minimize external dependencies

## Implementation Progress Tracking

### Phase 1: Essential Validation and Error Handling
1. **Request Validation**
   - [ ] Create `#[validate_request]` for automatic input validation on Axum handlers
   - [ ] Integrate with existing validation libraries like validator
   - [ ] Implement consistent error responses for validation failures
   
   *Updated at: Not started*

2. **Error Handling**
   - [ ] Create `#[api_handler]` macro for standardized error handling
   - [ ] Implement automatic conversion of internal errors to appropriate HTTP responses
   - [ ] Add request tracing and correlation IDs
   
   *Updated at: Not started*

3. **Authorization Checks**
   - [ ] Implement `#[require_permission("permission")]` for authorization enforcement
   - [ ] Support role-based and attribute-based access control
   - [ ] Add audit logging for authorization decisions
   
   *Updated at: Not started*

### Phase 2: Performance and Developer Experience 
1. **Rate Limiting**
   - [ ] Create simple `#[rate_limit]` attribute for endpoint-level rate limiting
   - [ ] Implement basic timeout and circuit breaking capabilities
   - [ ] Support IP-based and token-based rate limiting
   
   *Updated at: Not started*

2. **Logging and Instrumentation**
   - [ ] Implement `#[traced]` for automatic span creation and logging
   - [ ] Add performance metrics collection
   - [ ] Support structured logging with context propagation
   
   *Updated at: Not started*

### Phase 3: Axum Integration
1. **Routing Integration**
   - [ ] Simplify Axum route definitions with declarative attributes
   - [ ] Support middleware composition via attributes
   - [ ] Add response transformation helpers
   
   *Updated at: Not started*

2. **Middleware Integration**
   - [ ] Create middleware factory macros for common patterns
   - [ ] Implement middleware ordering utilities
   - [ ] Support conditional middleware application
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Feature Flag System

## Success Criteria
- Security requirements are consistently enforced across the application
- Developer productivity is improved with less boilerplate
- Performance overhead from declarative features is minimal
- Integration with Axum is seamless and intuitive
- Features work reliably without excessive dependencies

## Implementation Notes
This lightweight approach focuses on what provides the most value in a Rust Axum context, rather than trying to recreate Java/Spring patterns that don't align well with Rust's design philosophy. The implementation will use compile-time procedural macros to maintain Rust's performance characteristics.

### Example Implementation

```rust
// Example of the validation and error handling macros in use
#[api_handler]
#[validate_request]
async fn create_user(
    State(db): State<Arc<dyn DbService>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    #[validate] Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Authorization check (this would be handled by the macro)
    // require_permission("users.write")?;
    
    // Business logic here
    let user_id = db.create_user(&payload).await?;
    
    // Automatic result transformation by api_handler macro
    Ok(Json(UserCreatedResponse { id: user_id }))
}

// Example validation on a request struct
#[derive(Deserialize, Validate)]
struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,
    
    #[validate(email)]
    email: String,
}

// Example of a rate-limited endpoint
#[api_handler]
#[rate_limit(requests = 5, period = "1m")]
#[require_permission("reports.generate")]
async fn generate_report(
    State(report_svc): State<Arc<dyn ReportService>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(report_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let report = report_svc.generate(report_id).await?;
    Ok(Json(report))
}

// Example traced function with metrics
#[traced(level = "debug", metrics = true)]
async fn process_data(data: &[u8]) -> Result<ProcessedData, ProcessError> {
    // Processing logic
    // Spans and metrics are automatically collected
}
```

## References
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Rust Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Validator Crate](https://docs.rs/validator/latest/validator/) 