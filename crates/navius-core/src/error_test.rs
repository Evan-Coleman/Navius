// Integration tests for the error handling system
// These tests go beyond the unit tests in error.rs to test integration scenarios

use crate::error::{Error, ErrorCode, Result, ResultExt};
use std::io;
use std::path::Path;

// A mock function that simulates a database query that could fail
fn mock_database_query(should_fail: bool) -> Result<String> {
    if should_fail {
        Err(Error::database("Database connection failed"))
    } else {
        Ok("query result".to_string())
    }
}

// A mock function that simulates file operations that could fail
fn mock_file_operation(path: &Path) -> std::result::Result<String, io::Error> {
    // Simulate a file not found error
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("File not found: {}", path.display()),
    ))
}

// A function that simulates a chain of operations with error handling
fn perform_complex_operation(input: &str) -> Result<String> {
    // First step: validate input
    if input.is_empty() {
        return Err(Error::validation("Input cannot be empty"));
    }

    // Second step: try to read from a file (will fail)
    let file_path = Path::new("/non/existent/file.txt");
    // We don't need to capture the file_content as it will error out
    let _file_content = mock_file_operation(file_path).not_found(format!(
        "Could not find configuration file at {}",
        file_path.display()
    ))?;

    // This code won't be reached due to the error above
    let db_result = mock_database_query(false)?;

    Ok(format!("Processed: {} with {}", input, db_result))
}

// A function that uses with_context for detailed error messages
fn process_with_context(should_fail: bool) -> Result<()> {
    let result: std::result::Result<(), io::Error> = if should_fail {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Access denied",
        ))
    } else {
        Ok(())
    };

    result.with_context(ErrorCode::Authorization, || {
        format!(
            "Failed to access resource at {}",
            std::time::SystemTime::now().elapsed().unwrap().as_secs()
        )
    })
}

// A function that constructs a detailed error with additional context
fn create_detailed_error() -> Error {
    Error::validation("Username is invalid")
        .with_details(serde_json::json!({
            "field": "username",
            "validation": {
                "min_length": 3,
                "max_length": 20,
                "pattern": "^[a-zA-Z0-9_]+$"
            },
            "received": "u$er"
        }))
        .with_request_id("req-12345-abcde")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_propagation() {
        let result = perform_complex_operation("test_input");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.is_not_found());
        assert!(err.message.contains("configuration file"));

        // The error should have a source
        assert!(err.source.is_some());
    }

    #[test]
    fn test_error_with_context() {
        let result = process_with_context(true);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Authorization);
        assert!(err.message.contains("Failed to access resource"));

        // The error should have a source (the io::Error)
        assert!(err.source.is_some());
    }

    #[test]
    fn test_detailed_error_to_json() {
        let error = create_detailed_error();
        let json = error.to_json();

        assert_eq!(json["error"]["code"], "validation");
        assert_eq!(json["error"]["message"], "Username is invalid");
        assert_eq!(json["error"]["details"]["field"], "username");
        assert_eq!(json["error"]["details"]["received"], "u$er");
        assert_eq!(json["error"]["details"]["validation"]["min_length"], 3);
        assert_eq!(json["error"]["request_id"], "req-12345-abcde");
    }

    #[test]
    fn test_database_error_handling() {
        let result = mock_database_query(true);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::Database);
        assert_eq!(err.message, "Database connection failed");
    }

    #[test]
    fn test_different_error_conversions() {
        // Create an IO error
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");

        // Convert to different error types
        let validation_error: Result<()> = Err(io_error).validation("Invalid file permissions");

        let auth_error: Result<()> = Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Permission denied",
        ))
        .with_context(ErrorCode::Authorization, || "Unauthorized access");

        let not_found_error: Result<()> =
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
                .not_found("Resource not found");

        // Check error codes
        assert_eq!(validation_error.unwrap_err().code, ErrorCode::Validation);
        assert_eq!(auth_error.unwrap_err().code, ErrorCode::Authorization);
        assert_eq!(not_found_error.unwrap_err().code, ErrorCode::NotFound);
    }

    #[test]
    fn test_error_code_status_mapping() {
        // Test HTTP status code mapping
        assert_eq!(ErrorCode::Validation.status_code(), 400);
        assert_eq!(ErrorCode::Authentication.status_code(), 401);
        assert_eq!(ErrorCode::Authorization.status_code(), 403);
        assert_eq!(ErrorCode::NotFound.status_code(), 404);
        assert_eq!(ErrorCode::Conflict.status_code(), 409);
        assert_eq!(ErrorCode::Timeout.status_code(), 408);
        assert_eq!(ErrorCode::Internal.status_code(), 500);
        assert_eq!(ErrorCode::External.status_code(), 502);
        assert_eq!(ErrorCode::Database.status_code(), 500);
    }
}
