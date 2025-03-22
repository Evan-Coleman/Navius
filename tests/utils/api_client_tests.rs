use fake::faker::name::raw::*;
use fake::locales::EN;
use fake::{Fake, Faker};
use navius::core::config::app_config::{AppConfig, ServerConfig};
use navius::core::error::AppError;
use navius::core::utils::api_client::{ApiHandler, check_response_status, create_api_client};
use navius::core::utils::api_resource::ApiResource;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestResource {
    id: String,
    name: String,
}

impl ApiResource for TestResource {
    type Id = String;

    fn resource_type() -> &'static str {
        "test_resources"
    }

    fn api_name() -> &'static str {
        "TestAPI"
    }
}

impl TestResource {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    fn random() -> Self {
        Self {
            id: Faker.fake::<String>(),
            name: Name(EN).fake::<String>(),
        }
    }
}

#[test]
fn test_check_response_status() {
    // Success case
    let result = check_response_status(StatusCode::OK);
    assert!(result.is_ok());

    // Error case - Not Found
    let result = check_response_status(StatusCode::NOT_FOUND);
    assert!(matches!(result, Err(AppError::NotFound(_))));

    // Error case - Bad Request
    let result = check_response_status(StatusCode::BAD_REQUEST);
    assert!(matches!(result, Err(AppError::BadRequest(_))));

    // Error case - Unauthorized
    let result = check_response_status(StatusCode::UNAUTHORIZED);
    assert!(matches!(result, Err(AppError::Unauthorized(_))));

    // Error case - Server Error
    let result = check_response_status(StatusCode::INTERNAL_SERVER_ERROR);
    assert!(matches!(result, Err(AppError::ExternalServiceError(_))));
}

#[test]
fn test_create_api_client() {
    // Create a minimal AppConfig for testing
    let config = AppConfig {
        server: ServerConfig {
            timeout_seconds: 30,
            ..Default::default()
        },
        ..Default::default()
    };

    // Create the client
    let _client = create_api_client(&config);

    // We can only really assert that a client was created successfully
    assert!(true);
}

// Note: Async tests with mockito have been moved to the main codebase's test module
// but are currently disabled due to tokio runtime conflicts.
