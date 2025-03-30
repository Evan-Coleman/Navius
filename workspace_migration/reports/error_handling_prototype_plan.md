# Improved Error Handling Prototype Plan

**Date:** March 29, 2025  
**Status:** Planning  
**Priority:** High  
**Target Completion:** April 15, 2025

## Overview

This document outlines the plan for implementing an improved error handling system across the Navius framework. The goal is to create a consistent, context-rich error handling approach that maintains backward compatibility while providing enhanced features for debugging, logging, and user feedback.

## Objectives

1. Create a standardized error handling approach across all Navius crates
2. Improve error context propagation without introducing breaking changes
3. Implement better error categorization for appropriate HTTP status code mapping
4. Add utilities for error conversion and mapping between different error types
5. Enhance the developer and user experience for error scenarios

## Current Error Handling Analysis

The current error handling system has several limitations:

1. **Inconsistent Error Types**: Different crates use different error type structures
2. **Limited Context**: Many errors lack sufficient context for debugging
3. **Manual Conversion**: Error conversion between layers is often manual and inconsistent
4. **No Standardized Categorization**: Error categories are not standardized for HTTP responses
5. **Limited Tracing**: Error propagation lacks tracing across crate boundaries

## Implementation Approach

The improved error handling system will be implemented in phases to maintain compatibility:

### Phase 1: Core Error Types and Traits

1. **Define Error Context Trait**:
```rust
pub trait ErrorContext {
    /// Get the error context
    fn context(&self) -> &ErrorContextMap;
    
    /// Get a mutable reference to the error context
    fn context_mut(&mut self) -> &mut ErrorContextMap;
    
    /// Add context to the error
    fn with_context<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>;
        
    /// Get context value by key
    fn get_context<K>(&self, key: K) -> Option<&serde_json::Value>
    where
        K: AsRef<str>;
}
```

2. **Create Standardized Error Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaviusError {
    /// Error code
    pub code: ErrorCode,
    
    /// Error message
    pub message: String,
    
    /// Error source
    pub source: Option<String>,
    
    /// Error context
    pub context: ErrorContextMap,
    
    /// Error severity
    pub severity: ErrorSeverity,
    
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Request ID (if available)
    pub request_id: Option<String>,
}
```

3. **Define Error Code Enum**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Configuration,
    Validation,
    Authentication,
    Authorization,
    NotFound,
    Conflict,
    Internal,
    External,
    Timeout,
    Database,
    Cache,
    Plugin,
    Component,
    Serialization,
    Io,
    Unknown,
}
```

### Phase 2: Error Context Propagation

1. **Create Error Builder**:
```rust
pub struct ErrorBuilder {
    code: ErrorCode,
    message: String,
    source: Option<String>,
    context: ErrorContextMap,
    severity: ErrorSeverity,
}

impl ErrorBuilder {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
            context: ErrorContextMap::new(),
            severity: ErrorSeverity::from_code(code),
        }
    }
    
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
    
    pub fn with_context<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.context.insert(key.into(), value.into());
        self
    }
    
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
    
    pub fn build(self) -> NaviusError {
        NaviusError {
            code: self.code,
            message: self.message,
            source: self.source,
            context: self.context,
            severity: self.severity,
            timestamp: Utc::now(),
            request_id: None,  // Will be filled by middleware
        }
    }
}
```

2. **Implement Error Context Chain**:
```rust
pub trait ErrorChain {
    /// Create a new error that wraps this error
    fn wrap<E: std::error::Error + 'static>(self, error: E) -> Self;
    
    /// Chain with another error, preserving context
    fn chain<E: std::error::Error + ErrorContext + 'static>(self, error: E) -> Self;
    
    /// Get the full error chain
    fn chain_iter(&self) -> Box<dyn Iterator<Item = &(dyn std::error::Error + 'static)> + '_>;
    
    /// Get the context chain from all errors in the chain
    fn context_chain(&self) -> ErrorContextMap;
}
```

### Phase 3: Error Conversion System

1. **Implement From Traits**:
```rust
impl From<std::io::Error> for NaviusError {
    fn from(error: std::io::Error) -> Self {
        ErrorBuilder::new(ErrorCode::Io, error.to_string())
            .with_source("std::io")
            .with_context("kind", error.kind().to_string())
            .build()
    }
}

// Similar implementations for other common error types
```

2. **Create Error Extension Traits**:
```rust
pub trait ResultExt<T, E> {
    /// Convert any error to a NaviusError with the given code
    fn with_code(self, code: ErrorCode) -> Result<T, NaviusError>;
    
    /// Add context to the error
    fn with_context<K, V>(self, key: K, value: V) -> Result<T, NaviusError>
    where
        K: Into<String>,
        V: Into<serde_json::Value>;
        
    /// Set the error severity
    fn with_severity(self, severity: ErrorSeverity) -> Result<T, NaviusError>;
}

impl<T, E: std::error::Error + 'static> ResultExt<T, E> for Result<T, E> {
    // Implementation
}
```

### Phase 4: HTTP Integration and Middleware

1. **Implement HTTP Response Conversion**:
```rust
impl IntoResponse for NaviusError {
    fn into_response(self) -> Response {
        let status = match self.code {
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Validation => StatusCode::BAD_REQUEST,
            ErrorCode::Authentication => StatusCode::UNAUTHORIZED,
            ErrorCode::Authorization => StatusCode::FORBIDDEN,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::Timeout => StatusCode::REQUEST_TIMEOUT,
            // ... other mappings
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let body = json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "request_id": self.request_id,
                // Only include debug information in non-production
                "debug_info": if !cfg!(production) {
                    Some(json!({
                        "context": self.context,
                        "source": self.source,
                        "timestamp": self.timestamp,
                    }))
                } else {
                    None
                }
            }
        });
        
        (status, Json(body)).into_response()
    }
}
```

2. **Create Error Middleware**:
```rust
pub struct ErrorMiddleware<S> {
    inner: S,
}

impl<S> ErrorMiddleware<S> {
    pub fn new(service: S) -> Self {
        Self { inner: service }
    }
}

impl<S, B> Service<Request<B>> for ErrorMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
            
        let future = self.inner.call(req);

        Box::pin(async move {
            let response = future.await?;
            
            if let Some(error) = response.extensions().get::<NaviusError>() {
                let mut error = error.clone();
                if let Some(request_id) = request_id {
                    error.request_id = Some(request_id);
                }
                
                // Log the error if it's severe enough
                if error.severity >= ErrorSeverity::Medium {
                    error!(?error, "Request error");
                }
                
                Ok(error.into_response())
            } else {
                Ok(response)
            }
        })
    }
}
```

## Backward Compatibility Strategy

To maintain backward compatibility while implementing the improved error handling:

1. **Keep Existing Error Types**: Don't remove existing error types
2. **Implement From Traits**: Add From implementations to convert between old and new error types
3. **Add Extension Methods**: Extend existing Result types with conversion utilities
4. **Gradual Migration**: Update crates one at a time to use the new error handling

Example of backward compatibility layer:

```rust
// Keep existing AppError but implement conversions
impl From<AppError> for NaviusError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::NotFound(msg) => {
                ErrorBuilder::new(ErrorCode::NotFound, msg)
                    .with_source("app_error")
                    .build()
            }
            // ... other conversions
        }
    }
}

impl From<NaviusError> for AppError {
    fn from(error: NaviusError) -> Self {
        match error.code {
            ErrorCode::NotFound => AppError::NotFound(error.message),
            // ... other conversions
            _ => AppError::InternalServerError(error.message),
        }
    }
}
```

## Testing Strategy

The error handling system will be rigorously tested:

1. **Unit Tests**: Test each component of the error system in isolation
2. **Integration Tests**: Test error propagation across crate boundaries
3. **Conversion Tests**: Test automatic conversions between error types
4. **Context Propagation Tests**: Test that context is properly preserved
5. **HTTP Response Tests**: Test mapping to appropriate HTTP responses
6. **Performance Tests**: Ensure the error handling system doesn't introduce significant overhead

## Implementation Schedule

1. **Week 1 (April 1-7, 2025)**:
   - Define core error types and traits
   - Implement error context propagation
   - Create error conversion system

2. **Week 2 (April 8-15, 2025)**:
   - Implement HTTP integration and middleware
   - Add backward compatibility layer
   - Create tests and documentation
   - Integrate with selected crates for validation

## Expected Outcomes

The improved error handling system will provide:

1. **Consistent Error Types**: Standardized error structure across all crates
2. **Rich Context**: Errors with detailed context for debugging
3. **Automatic Conversion**: Simplified error conversion between crates
4. **Standardized Categorization**: Consistent error categorization and HTTP mapping
5. **Error Tracing**: Comprehensive tracing of errors across crate boundaries
6. **Improved Developer Experience**: Better tooling for error handling and debugging
7. **Enhanced User Experience**: More informative and appropriate error responses

## Conclusion

The improved error handling prototype will significantly enhance the reliability and developer experience of the Navius framework. By implementing a consistent, context-rich error handling approach that maintains backward compatibility, we will address one of the key issues identified in the design evaluations while setting the stage for better error handling across the entire ecosystem. 