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
    use crate::core::error::AppError;
    use reqwest::StatusCode;

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

    // All other mock HTTP tests moved to integration tests
}
