use crate::core::config::app_config::AppConfig;
use crate::core::error::AppError;
use crate::core::utils::api_resource::ApiResource;
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument, warn};

/// Creates a new API client with the given configuration
pub fn create_api_client(config: &AppConfig) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(config.server.timeout_seconds))
        .build()
        .unwrap_or_else(|_| {
            warn!("Failed to build custom HTTP client, using default");
            Client::new()
        })
}

/// API handler for making requests to external services
#[derive(Clone)]
pub struct ApiHandler {
    client: Client,
    base_url: String,
}

impl ApiHandler {
    /// Create a new API handler
    pub fn new(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    /// Make a GET request to the given path
    #[instrument(skip(self), level = "debug")]
    pub async fn get<T: DeserializeOwned + ApiResource>(&self, path: &str) -> Result<T, AppError> {
        let url = format!("{}{}", self.base_url, path);
        debug!("Making GET request to {}", url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Request failed: {}", e);
            AppError::ExternalServiceError(format!("Request to {} failed: {}", url, e))
        })?;

        self.process_response::<T>(response).await
    }

    /// Process the HTTP response
    async fn process_response<T: DeserializeOwned + ApiResource>(
        &self,
        response: Response,
    ) -> Result<T, AppError> {
        let status = response.status();
        check_response_status(status)?;

        let body = response.text().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            AppError::ExternalServiceError(format!("Failed to read response body: {}", e))
        })?;

        serde_json::from_str::<T>(&body).map_err(|e| {
            error!("Failed to parse response: {}", e);
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })
    }
}

/// Check the response status and return an error if not OK
pub fn check_response_status(status: StatusCode) -> Result<(), AppError> {
    match status {
        StatusCode::OK => Ok(()),
        StatusCode::NOT_FOUND => Err(AppError::NotFound("Resource not found".to_string())),
        StatusCode::BAD_REQUEST => Err(AppError::BadRequest("Bad request".to_string())),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            Err(AppError::Unauthorized("Unauthorized".to_string()))
        }
        _ if status.is_server_error() => Err(AppError::ExternalServiceError(format!(
            "Server error: {}",
            status
        ))),
        _ => Err(AppError::ExternalServiceError(format!(
            "Unexpected status code: {}",
            status
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::{AppConfig, ServerConfig};
    use crate::core::utils::api_resource::ApiResource;
    use serde::{Deserialize, Serialize};
    use std::env;

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

        // We can only really test that a client was created and didn't panic
        // There's no great way to test its configuration without making a request
        assert!(true);
    }

    #[test]
    fn test_check_response_status() {
        // Test OK status
        let result = check_response_status(StatusCode::OK);
        assert!(result.is_ok());

        // Test NOT_FOUND status
        let result = check_response_status(StatusCode::NOT_FOUND);
        assert!(result.is_err());
        if let AppError::NotFound(_) = result.unwrap_err() {
            // This is expected
        } else {
            panic!("Expected NotFound error");
        }

        // Test BAD_REQUEST status
        let result = check_response_status(StatusCode::BAD_REQUEST);
        assert!(result.is_err());
        if let AppError::BadRequest(_) = result.unwrap_err() {
            // This is expected
        } else {
            panic!("Expected BadRequest error");
        }

        // Test UNAUTHORIZED status
        let result = check_response_status(StatusCode::UNAUTHORIZED);
        assert!(result.is_err());
        if let AppError::Unauthorized(_) = result.unwrap_err() {
            // This is expected
        } else {
            panic!("Expected Unauthorized error");
        }

        // Test FORBIDDEN status
        let result = check_response_status(StatusCode::FORBIDDEN);
        assert!(result.is_err());
        if let AppError::Unauthorized(_) = result.unwrap_err() {
            // This is expected
        } else {
            panic!("Expected Unauthorized error");
        }

        // Test server error
        let result = check_response_status(StatusCode::INTERNAL_SERVER_ERROR);
        assert!(result.is_err());
        if let AppError::ExternalServiceError(_) = result.unwrap_err() {
            // This is expected
        } else {
            panic!("Expected ExternalServiceError error");
        }

        // Test other error
        let result = check_response_status(StatusCode::IM_A_TEAPOT);
        assert!(result.is_err());
        if let AppError::ExternalServiceError(_) = result.unwrap_err() {
            // This is expected
        } else {
            panic!("Expected ExternalServiceError error");
        }
    }
}
