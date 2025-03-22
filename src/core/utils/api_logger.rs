//! # API Logger
//!
//! This module provides user-extensible logging utilities for API operations.
//! The core implementation is in the core/utils/api_logger module.

use crate::core::config::app_config::AppConfig;
use crate::core::error::AppError;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing::{info, warn};

#[derive(Debug)]
pub struct RequestLogger {
    pub request_id: String,
    pub api_name: String,
    pub resource_type: String,
}

pub fn log_request(logger: &RequestLogger, id: impl std::fmt::Display) {
    info!(
        request_id = %logger.request_id,
        api = %logger.api_name,
        resource = %logger.resource_type,
        resource_id = %id,
        "Fetching resource"
    );
}

pub fn log_response<T: Serialize>(logger: &RequestLogger, _response: &T) {
    info!(
        request_id = %logger.request_id,
        api = %logger.api_name,
        resource = %logger.resource_type,
        "Resource fetched successfully"
    );
}

pub async fn api_call<F, Fut, T, E, I>(
    api_name: &str,
    _url: &str,
    fetch_fn: F,
    resource_type: &str,
    resource_id: I,
) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<reqwest::Response, E>>,
    T: for<'de> Deserialize<'de> + Serialize,
    E: std::error::Error + Send + Sync + 'static,
    I: std::fmt::Display,
{
    let logger = RequestLogger {
        request_id: uuid::Uuid::new_v4().to_string(),
        api_name: api_name.to_string(),
        resource_type: resource_type.to_string(),
    };

    log_request(&logger, resource_id);

    let response = fetch_fn()
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    let result = response
        .json::<T>()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse response: {}", e)))?;

    log_response(&logger, &result);

    Ok(result)
}

/// Check the response status code and return an appropriate Result
pub fn check_response_status(status: StatusCode) -> Result<(), AppError> {
    match status {
        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => Ok(()),
        StatusCode::NOT_FOUND => Err(AppError::NotFound("Resource not found".to_string())),
        StatusCode::BAD_REQUEST => Err(AppError::BadRequest("Invalid request".to_string())),
        StatusCode::UNAUTHORIZED => Err(AppError::Unauthorized("Unauthorized".to_string())),
        _ => Err(AppError::ExternalServiceError(format!(
            "Unexpected status code: {}",
            status
        ))),
    }
}

/// Create a new API client with the specified configuration
pub fn create_api_client(config: &AppConfig) -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(
            config.server.timeout_seconds,
        ))
        .build()
        .unwrap_or_else(|_| Client::new())
}

// Add your custom API logging utilities below
//
// Example:
//
// /// Log specialized API metrics
// pub fn log_api_metrics(
//     api_name: &str,
//     endpoint: &str,
//     duration_ms: u64,
//     status_code: u16,
// ) {
//     info!(
//         "📊 API metrics - {}: endpoint={}, duration={}ms, status={}",
//         api_name, endpoint, duration_ms, status_code
//     );
//
//     // Record metrics
//     metrics::histogram!(
//         "api_request_duration_ms",
//         duration_ms as f64,
//         "api" => api_name.to_string(),
//         "endpoint" => endpoint.to_string()
//     );
//
//     metrics::counter!(
//         "api_requests_total",
//         1,
//         "api" => api_name.to_string(),
//         "endpoint" => endpoint.to_string(),
//         "status" => status_code.to_string()
//     );
// }
