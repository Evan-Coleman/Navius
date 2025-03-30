// Simple tests for the navius-core error handling system
// These tests are minimal and avoid dependencies on other parts of the codebase

use navius_core::error::{Error, ErrorCode};
use navius_test::error::{TestResult, assert_eq, assert_true};
use std::error::Error as StdError;

#[test]
fn test_create_basic_error() -> TestResult<()> {
    let error = Error::new(ErrorCode::Validation, "Invalid input");
    assert_eq(
        error.code,
        ErrorCode::Validation,
        "Error code should be Validation",
    )?;
    assert_eq(error.message, "Invalid input", "Error message should match")?;
    assert_true(error.source.is_none(), "Error source should be None")?;

    Ok(())
}

#[test]
fn test_error_factory_methods() -> TestResult<()> {
    let validation_error = Error::validation("Invalid input");
    assert_eq(
        validation_error.code,
        ErrorCode::Validation,
        "Validation error should have Validation code",
    )?;

    let not_found_error = Error::not_found("Resource not found");
    assert_eq(
        not_found_error.code,
        ErrorCode::NotFound,
        "Not found error should have NotFound code",
    )?;

    let internal_error = Error::internal("System error");
    assert_eq(
        internal_error.code,
        ErrorCode::Internal,
        "Internal error should have Internal code",
    )?;

    Ok(())
}

#[test]
fn test_error_with_details() -> TestResult<()> {
    let error = Error::validation("Invalid input").with_details(serde_json::json!({
        "field": "username",
        "reason": "too short"
    }));

    assert_true(error.details.is_some(), "Error should have details")?;
    assert_eq(
        error.details.as_ref().unwrap()["field"],
        "username",
        "Error details should contain the field name",
    )?;

    Ok(())
}

#[test]
fn test_error_with_request_id() -> TestResult<()> {
    let error = Error::validation("Invalid input").with_request_id("req-123");

    assert_true(error.request_id.is_some(), "Error should have request ID")?;
    assert_eq(
        error.request_id.unwrap(),
        "req-123",
        "Request ID should match",
    )?;

    Ok(())
}

#[test]
fn test_error_to_json() -> TestResult<()> {
    let error = Error::validation("Invalid input")
        .with_details(serde_json::json!({"field": "username"}))
        .with_request_id("req-123");

    let json = error.to_json();
    assert_eq(
        json["error"]["code"],
        "validation",
        "JSON error code should be 'validation'",
    )?;
    assert_eq(
        json["error"]["message"],
        "Invalid input",
        "JSON error message should match",
    )?;
    assert_eq(
        json["error"]["status"],
        400,
        "JSON error status should be 400",
    )?;
    assert_eq(
        json["error"]["details"]["field"],
        "username",
        "JSON error details should contain field",
    )?;
    assert_eq(
        json["error"]["request_id"],
        "req-123",
        "JSON error request ID should match",
    )?;

    Ok(())
}

#[test]
fn test_error_code_status() -> TestResult<()> {
    assert_eq(
        ErrorCode::Validation.status_code(),
        400,
        "Validation status code should be 400",
    )?;
    assert_eq(
        ErrorCode::Authentication.status_code(),
        401,
        "Authentication status code should be 401",
    )?;
    assert_eq(
        ErrorCode::NotFound.status_code(),
        404,
        "NotFound status code should be 404",
    )?;
    assert_eq(
        ErrorCode::Conflict.status_code(),
        409,
        "Conflict status code should be 409",
    )?;
    assert_eq(
        ErrorCode::Internal.status_code(),
        500,
        "Internal status code should be 500",
    )?;

    Ok(())
}
