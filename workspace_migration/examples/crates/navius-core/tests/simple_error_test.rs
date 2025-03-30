// Simple tests for the navius-core error handling system
// These tests are minimal and avoid dependencies on other parts of the codebase

use navius_core::error::{Error, ErrorCode};
use std::error::Error as StdError;

#[test]
fn test_create_basic_error() {
    let error = Error::new(ErrorCode::Validation, "Invalid input");
    assert_eq!(error.code, ErrorCode::Validation);
    assert_eq!(error.message, "Invalid input");
    assert!(error.source.is_none());
}

#[test]
fn test_error_factory_methods() {
    let validation_error = Error::validation("Invalid input");
    assert_eq!(validation_error.code, ErrorCode::Validation);

    let not_found_error = Error::not_found("Resource not found");
    assert_eq!(not_found_error.code, ErrorCode::NotFound);

    let internal_error = Error::internal("System error");
    assert_eq!(internal_error.code, ErrorCode::Internal);
}

#[test]
fn test_error_with_details() {
    let error = Error::validation("Invalid input").with_details(serde_json::json!({
        "field": "username",
        "reason": "too short"
    }));

    assert!(error.details.is_some());
    assert_eq!(error.details.as_ref().unwrap()["field"], "username");
}

#[test]
fn test_error_with_request_id() {
    let error = Error::validation("Invalid input").with_request_id("req-123");

    assert!(error.request_id.is_some());
    assert_eq!(error.request_id.unwrap(), "req-123");
}

#[test]
fn test_error_to_json() {
    let error = Error::validation("Invalid input")
        .with_details(serde_json::json!({"field": "username"}))
        .with_request_id("req-123");

    let json = error.to_json();
    assert_eq!(json["error"]["code"], "validation");
    assert_eq!(json["error"]["message"], "Invalid input");
    assert_eq!(json["error"]["status"], 400);
    assert_eq!(json["error"]["details"]["field"], "username");
    assert_eq!(json["error"]["request_id"], "req-123");
}

#[test]
fn test_error_code_status() {
    assert_eq!(ErrorCode::Validation.status_code(), 400);
    assert_eq!(ErrorCode::Authentication.status_code(), 401);
    assert_eq!(ErrorCode::NotFound.status_code(), 404);
    assert_eq!(ErrorCode::Conflict.status_code(), 409);
    assert_eq!(ErrorCode::Internal.status_code(), 500);
}
