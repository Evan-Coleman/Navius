# Data Validation Roadmap

## Overview
A pragmatic, security-first approach to data validation for Navius that leverages existing validation libraries while providing consistent validation patterns throughout the application.

## Current State
Our application needs structured input validation to ensure data integrity and security, particularly for API endpoints exposed to external users.

## Target State
A lightweight but effective validation system that:
- Ensures all inputs are properly validated for security
- Integrates seamlessly with Axum's extractors
- Provides consistent error responses
- Balances security with developer productivity
- Is independent of specific authentication providers

## Implementation Progress Tracking

### Phase 1: Core Security Validation
1. **Request Data Validation**
   - [ ] Integrate with validator crate for struct-level validation
   - [ ] Implement custom Axum extractor for validated JSON/Form data
   - [ ] Create security-focused validators for sensitive fields
   
   *Updated at: Not started*

2. **Input Sanitization**
   - [ ] Create helpers for safe parameter handling
   - [ ] Implement injection prevention validation
   - [ ] Add validation for specific data types
   
   *Updated at: Not started*

### Phase 2: Developer Experience
1. **Standardized Error Responses**
   - [ ] Create unified validation error format
   - [ ] Implement consistent error responses for all validation failures
   - [ ] Add detailed context for validation errors
   
   *Updated at: Not started*

2. **Common Validation Patterns**
   - [ ] Build reusable validation functions for common use cases
   - [ ] Implement validators for business domain objects
   - [ ] Create validation test helpers
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 22, 2025
- **Next Milestone**: Schema Validation System

## Success Criteria
- All external inputs are properly validated
- Security vulnerabilities from invalid input are prevented
- Validation errors provide clear guidance to API clients
- Developer experience is improved with consistent validation patterns
- Performance overhead is minimized

## Implementation Notes
This approach focuses on practical validation that leverages existing tools like the validator crate and Axum's extractor system rather than building a complex custom validation framework. We'll prioritize security-critical validations while maintaining simplicity and developer productivity.

Authentication-specific validation (Entra tokens, AWS resource validation) has been consolidated in the AWS Integration roadmap to avoid duplication.

### Example Implementation

```rust
use axum::{
    extract::{FromRequest, RequestParts, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use validator::{Validate, ValidationErrors};
use std::sync::Arc;

// Example validated request model with security-focused validations
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8), custom = "validate_password_strength")]
    pub password: String,

    #[validate(custom = "validate_organization_id")]
    pub organization_id: Option<String>,
}

// Custom validator for password strength
fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    // Check for minimum complexity requirements
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !(has_uppercase && has_lowercase && has_digit && has_special) {
        let mut error = validator::ValidationError::new("password_complexity");
        error.message = Some("Password must contain uppercase, lowercase, digit, and special characters".into());
        return Err(error);
    }

    Ok(())
}

// Custom validator that checks organization access
fn validate_organization_id(org_id: &str) -> Result<(), validator::ValidationError> {
    // This would typically check against the current user's permissions
    // Implementation would be authentication-agnostic
    if org_id.len() != 36 {  // UUID validation
        let mut error = validator::ValidationError::new("invalid_format");
        error.message = Some("Organization ID must be a valid UUID".into());
        return Err(error);
    }
    
    Ok(())
}

// Standardized validation error response
#[derive(Debug, serde::Serialize)]
pub struct ValidationErrorResponse {
    pub message: String,
    pub errors: std::collections::HashMap<String, Vec<String>>,
}

impl IntoResponse for ValidationErrors {
    fn into_response(self) -> Response {
        let mut errors = std::collections::HashMap::new();
        
        for (field, field_errors) in self.field_errors() {
            let error_messages: Vec<String> = field_errors
                .iter()
                .map(|error| {
                    error.message.clone()
                        .unwrap_or_else(|| "Invalid value".into())
                        .to_string()
                })
                .collect();
            
            errors.insert(field.to_string(), error_messages);
        }
        
        let error_response = ValidationErrorResponse {
            message: "Validation failed".to_string(),
            errors,
        };
        
        (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
    }
}

// Custom extractor that validates input
pub struct ValidatedJson<T>(pub T);

#[axum::async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: Validate + serde::de::DeserializeOwned,
    B: Send + Sync,
{
    type Rejection = Response;
    
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the JSON payload
        let Json(value) = Json::<T>::from_request(req)
            .await
            .map_err(|e| {
                let error_message = format!("Invalid JSON: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ValidationErrorResponse {
                        message: error_message,
                        errors: std::collections::HashMap::new(),
                    }),
                )
                    .into_response()
            })?;
        
        // Validate the data
        value.validate().map_err(|e| e.into_response())?;
        
        Ok(ValidatedJson(value))
    }
}

// Example handler using the validated extractor
async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
    State(db): State<Arc<DbPool>>,
) -> impl IntoResponse {
    // Business logic here - we know the data is valid
    // No need for additional validation checks
    
    // Create the user
    match db.create_user(&payload).await {
        Ok(user_id) => {
            (StatusCode::CREATED, Json(json!({ "id": user_id }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

// Context-aware validation middleware
async fn validate_context<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // This middleware validates request context in a way that's
    // independent of specific authentication providers
    
    // Get user identity from request extensions (set by auth middleware)
    let user_identity = request.extensions()
        .get::<UserIdentity>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Extract the target resource ID from the request
    let resource_id = extract_resource_id(&request)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // Validate access based on identity and resource
    if !validate_access(user_identity, resource_id).await {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Continue with the request
    Ok(next.run(request).await)
}

// Helper function to extract resource ID from request
fn extract_resource_id<B>(request: &Request<B>) -> Option<String> {
    // Extract from path parameters, query params, or body
    // Implementation depends on your routing structure
    None  // Placeholder
}

// Helper function to validate access
async fn validate_access(identity: &UserIdentity, resource_id: &str) -> bool {
    // Check if the user has access to the resource
    // This could call a permission service or check against a database
    true  // Placeholder
}

// Generic user identity that's not tied to specific auth providers
#[derive(Debug, Clone)]
struct UserIdentity {
    pub user_id: String,
    pub roles: Vec<String>,
    pub org_id: String,
}
```

## References
- [validator crate](https://docs.rs/validator/latest/validator/)
- [Axum extractors](https://docs.rs/axum/latest/axum/extract/index.html)
- [OWASP Input Validation Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html) 