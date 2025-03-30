# Navius Enhanced Error Handling System

This crate demonstrates the improved error handling system for the Navius framework, which provides a consistent, user-friendly approach to error management across all components of the application.

## Key Features

- **Standardized Error Codes**: Each error belongs to a specific category (Validation, Authentication, Authorization, etc.) with corresponding HTTP status codes
- **Detailed Context**: Errors can include detailed context information via structured JSON
- **Source Error Tracking**: Original errors are preserved for debugging and tracing
- **Consistent Error Format**: All errors are presented in a unified JSON format for API responses
- **Request ID Support**: Optional request ID for correlation in logs and debugging
- **Intuitive Helper Methods**: Factory methods and extension traits make error creation and conversion easy

## Error Types

The system defines the following error types:

| Error Code | HTTP Status | Use Case |
|------------|-------------|----------|
| Validation | 400 | Input validation errors |
| Authentication | 401 | Failed login attempts |
| Authorization | 403 | Permission denied errors |
| NotFound | 404 | Resource not found |
| Conflict | 409 | Resource conflicts (e.g., duplicate entries) |
| Timeout | 408 | Operation timeouts |
| Internal | 500 | Internal server errors |
| External | 502 | External service errors |
| Database | 500 | Database errors |
| Cache | 500 | Cache-related errors |
| Plugin | 500 | Plugin system errors |
| Component | 500 | Component lifecycle errors |
| Configuration | 500 | Configuration errors |
| Serialization | 400 | Serialization/deserialization errors |
| IO | 500 | Input/output errors |
| Unknown | 500 | Unclassified errors |

## Usage Examples

### Basic Error Creation

```rust
// Create a basic validation error
let error = Error::validation("Username must be at least 3 characters long");

// Create an error with additional details
let error = Error::validation("Invalid input")
    .with_details(json!({
        "field": "username",
        "min_length": 3,
        "actual_length": 2
    }))
    .with_request_id("req-12345");
```

### Error Conversion

```rust
// Convert a standard io::Error to a Navius error
let file_result = std::fs::read_to_string("/path/to/file");

// Convert to a NotFound error with context
let result = file_result.not_found("Configuration file not found");

// Or use with_context for more control
let result = file_result.with_context(ErrorCode::Configuration, || {
    format!("Failed to load configuration at {}", std::time::SystemTime::now())
});
```

### API Response Format

Errors can be easily converted to a standardized API response:

```json
{
  "error": {
    "code": "validation",
    "message": "Username must be at least 3 characters long",
    "status": 400,
    "details": {
      "field": "username",
      "min_length": 3,
      "actual_length": 2
    },
    "request_id": "req-12345"
  }
}
```

## Demo Application

Run the REST API demo to see the error handling system in action:

```
cargo run --example rest_api_demo
```

The demo showcases how the error handling system handles different scenarios and produces consistent, user-friendly error responses.

## Benefits

1. **Consistency**: All errors follow the same format and structure
2. **Context Preservation**: Original error information is preserved for debugging
3. **Code Organization**: Clean separation between error handling and business logic
4. **User-Friendly**: Error messages are clear and informative
5. **API-Ready**: Errors can be directly converted to API responses with appropriate HTTP status codes
6. **Extensible**: New error types can be easily added when needed
7. **Fine-Grained Control**: Allows different handling based on error type

## Implementation in Navius Framework

This error handling system has been integrated into the core Navius framework, ensuring consistent error handling across all crates and components. 