//! Reliability features
//!
//! This module provides middleware components for enhancing application resilience:
//! - Retry mechanisms
//! - Circuit breakers
//! - Rate limiting
//! - Concurrency control
//! - Request timeouts

pub mod circuit_breaker;
pub mod concurrency;
pub mod metrics;
pub mod rate_limit;
pub mod retry;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::router::AppState;
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_timeout_layer() {
        let config = TimeoutConfig {
            enabled: true,
            timeout_seconds: 1,
        };

        let layer = build_timeout_layer(&config).unwrap();
        let app = Router::new()
            .route(
                "/",
                axum::routing::get(|| async {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    "OK"
                }),
            )
            .layer(layer)
            .with_state(Arc::new(AppState::default()));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }

    // Add other test cases here...
}

// Re-export key components
pub use circuit_breaker::CircuitBreakerConfig;
pub use concurrency::ConcurrencyLimitConfig;
pub use rate_limit::RateLimitConfig;
pub use retry::RetryConfig;

use axum::Router;
use axum::body::HttpBody;
use std::sync::Arc;
use std::time::Duration;
use tower::{BoxError, Layer, Service, ServiceBuilder};
use tower_http::timeout::TimeoutLayer;
use tracing::{info, warn};

use crate::core::config::app_config::{
    CircuitBreakerConfig, ConcurrencyConfig, RateLimitConfig, ReliabilityConfig, RetryConfig,
    TimeoutConfig,
};
use crate::core::error::{ErrorResponse, ErrorType};
use crate::core::router::AppState;
use crate::core::utils::request_id::get_req_id;

/// Apply reliability middleware to the router based on configuration
pub fn apply_reliability(
    router: Router<Arc<AppState>>,
    config: &ReliabilityConfig,
) -> Result<Router<Arc<AppState>>, AppError> {
    let mut service_builder = ServiceBuilder::new();

    // Configure timeout layer
    if let Some(layer) = build_timeout_layer(&config.timeout)? {
        service_builder = service_builder.layer(layer);
    }

    // Configure circuit breaker
    if let Some(layer) = build_circuit_breaker_layer(&config.circuit_breaker)? {
        service_builder = service_builder.layer(layer);
    }

    // Configure rate limiting
    if let Some(layer) = build_rate_limit_layer(&config.rate_limit)? {
        service_builder = service_builder.layer(layer);
    }

    // Configure concurrency limits
    if let Some(layer) = build_concurrency_layer(&config.concurrency)? {
        service_builder = service_builder.layer(layer);
    }

    Ok(router.layer(service_builder))
}

/// Build the retry layer based on configuration
fn build_retry_layer(config: &RetryConfig) -> Result<Option<RetryLayer>, AppError> {
    if !config.enabled {
        info!("Request retries are disabled");
        return Ok(None);
    }

    // Validate retry configuration
    if config.max_attempts < 1 {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            "Retry max_attempts must be at least 1".to_string(),
        )));
    }

    if config.base_delay_ms == 0 {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            "Retry base_delay_ms must be greater than 0".to_string(),
        )));
    }

    info!(
        "Configuring request retries: max_attempts={}, base_delay={}ms, exponential_backoff={}",
        config.max_attempts, config.base_delay_ms, config.use_exponential_backoff
    );

    Ok(Some(RetryLayer::new(
        config.max_attempts,
        Duration::from_millis(config.base_delay_ms),
        Duration::from_millis(config.max_delay_ms),
        config.use_exponential_backoff,
        config.retry_status_codes.clone(),
    )))
}

/// Build the circuit breaker layer based on configuration
fn build_circuit_breaker_layer(
    config: &CircuitBreakerConfig,
) -> Result<Option<circuit_breaker::CircuitBreakerLayer>, AppError> {
    if !config.enabled {
        info!("Circuit breaker is disabled");
        return Ok(None);
    }

    if config.use_consecutive_failures {
        info!(
            "Configuring circuit breaker (consecutive failures mode): threshold={}, reset_timeout={}ms, success_threshold={}",
            config.failure_threshold, config.reset_timeout_ms, config.success_threshold
        );
    } else {
        info!(
            "Configuring circuit breaker (rolling window mode): window={}s, failure_percentage={}%, reset_timeout={}ms, success_threshold={}",
            config.window_seconds,
            config.failure_percentage,
            config.reset_timeout_ms,
            config.success_threshold
        );
    }

    Ok(Some(circuit_breaker::CircuitBreakerLayer::new_with_config(
        config.failure_threshold,
        Duration::from_millis(config.reset_timeout_ms),
        config.success_threshold,
        config.window_seconds,
        config.failure_percentage,
        config.use_consecutive_failures,
        config.failure_status_codes.clone(),
    )))
}

/// Build the rate limiting layer based on configuration
fn build_rate_limit_layer(
    config: &RateLimitConfig,
) -> Result<Option<rate_limit::RateLimitLayer>, AppError> {
    if !config.enabled {
        info!("Rate limiting is disabled");
        return Ok(None);
    }

    if config.requests_per_window == 0 {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            "Invalid rate limit configuration: requests_per_window cannot be zero".to_string(),
        )));
    }

    info!(
        "Configuring rate limiter: {} requests per {} seconds, per_client={}",
        config.requests_per_window, config.window_seconds, config.per_client
    );

    Ok(Some(rate_limit::RateLimitLayer::new(
        config.requests_per_window,
        Duration::from_secs(config.window_seconds),
        config.per_client,
    )))
}

/// Build the timeout layer based on configuration
fn build_timeout_layer(config: &TimeoutConfig) -> Result<Option<TimeoutLayer>, AppError> {
    if !config.enabled {
        info!("Request timeouts are disabled");
        return Ok(None);
    }

    // Validate timeout configuration
    if config.timeout_seconds == 0 {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            "Timeout seconds must be greater than 0".to_string(),
        )));
    }

    if config.timeout_seconds > MAX_TIMEOUT_SECONDS {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            format!(
                "Timeout exceeds maximum allowed duration of {} seconds",
                MAX_TIMEOUT_SECONDS
            ),
        )));
    }

    info!("Configuring request timeouts: {}s", config.timeout_seconds);
    let timeout = Duration::from_secs(config.timeout_seconds);

    Ok(Some(TimeoutLayer::new(timeout).on_timeout(|| async {
        let error = ErrorResponse::new(ErrorType::RequestTimeout, "Request timed out".to_string())
            .with_request_id(get_req_id())
            .with_details(json!({"timeout_seconds": config.timeout_seconds}));

        // Log with structured logging
        warn!(
            error_code = error.error.code,
            error_message = error.error.message,
            timeout_seconds = config.timeout_seconds,
            "Request timeout occurred"
        );

        error.into_response()
    })))
}

/// Build the concurrency layer based on configuration
fn build_concurrency_layer(
    config: &ConcurrencyConfig,
) -> Result<Option<concurrency::ConcurrencyLimitLayer>, AppError> {
    if !config.enabled {
        info!("Concurrency limits are disabled");
        return Ok(None);
    }

    if config.max_concurrent_requests == 0 {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            "Concurrency limit must be greater than 0".to_string(),
        )));
    }

    if config.max_concurrent_requests > MAX_CONCURRENT_REQUESTS {
        return Err(AppError::new(ErrorResponse::new(
            ErrorType::ConfigurationError,
            format!(
                "Concurrency limit exceeds maximum allowed value of {}",
                MAX_CONCURRENT_REQUESTS
            ),
        )));
    }

    info!(
        "Configuring concurrency limits: max={}",
        config.max_concurrent_requests
    );

    Ok(Some(concurrency::ConcurrencyLimitLayer::new(
        config.max_concurrent_requests,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{config::app_config::AppConfig, error::AppError};
    use axum::{body::Body, http::Request, routing::get};
    use std::sync::Arc;
    use tower::ServiceExt;

    // Enhanced test with error handling verification
    #[tokio::test]
    async fn test_full_reliability_stack() -> Result<(), AppError> {
        let config = ReliabilityConfig {
            timeout: TimeoutConfig {
                enabled: true,
                timeout_seconds: 1,
            },
            circuit_breaker: CircuitBreakerConfig::default(),
            rate_limit: RateLimitConfig::default(),
            concurrency: ConcurrencyConfig::default(),
            retry: RetryConfig::default(),
        };

        let router = apply_reliability(
            Router::new().route(
                "/",
                get(|| async {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    Result::<_, AppError>::Ok("OK")
                }),
            ),
            &config,
        )?;

        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);

        // Verify error response format
        let body = hyper::body::to_bytes(response.into_body()).await?;
        let error_response: serde_json::Value = serde_json::from_slice(&body)?;

        assert_eq!(error_response["error"]["code"], "REQUEST_TIMEOUT");
        assert_eq!(error_response["error"]["message"], "Request timed out");

        Ok(())
    }

    // Add more tests for each middleware component

    // Update test to verify error formatting
    #[tokio::test]
    async fn test_error_formatting() -> Result<(), AppError> {
        let config = ReliabilityConfig {
            timeout: TimeoutConfig {
                enabled: true,
                timeout_seconds: 1,
            },
            ..Default::default()
        };

        let router = apply_reliability(
            Router::new().route(
                "/",
                get(|| async {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    Ok::<_, AppError>("OK")
                }),
            ),
            &config,
        )?;

        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;

        let body = hyper::body::to_bytes(response.into_body()).await?;
        let error_response: ErrorResponse = serde_json::from_slice(&body)?;

        assert_eq!(error_response.error.code, "REQUEST_TIMEOUT");
        assert_eq!(error_response.error.message, "Request timed out");
        assert_eq!(error_response.error.details.unwrap()["timeout_seconds"], 1);

        Ok(())
    }

    // Add concurrency limit error handling
    #[tokio::test]
    async fn test_concurrency_limit() -> Result<(), AppError> {
        let config = ReliabilityConfig {
            concurrency: ConcurrencyConfig {
                enabled: true,
                max_concurrent_requests: 1,
            },
            ..Default::default()
        };

        let (tx, rx) = tokio::sync::oneshot::channel();

        let router = apply_reliability(
            Router::new().route(
                "/",
                get(|| async {
                    rx.await.unwrap();
                    Ok::<_, AppError>("OK")
                }),
            ),
            &config,
        )?;

        // First request will block
        let request1 = router
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty())?);

        // Second request should fail
        let response2 = router
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;

        assert_eq!(response2.status(), StatusCode::TOO_MANY_REQUESTS);

        // Verify error details
        let body = hyper::body::to_bytes(response2.into_body()).await?;
        let error_response: ErrorResponse = serde_json::from_slice(&body)?;

        assert_eq!(error_response.error.code, "TOO_MANY_REQUESTS");
        assert!(
            error_response.error.details.unwrap()["max_connections"]
                .as_u64()
                .unwrap()
                > 0
        );

        // Cleanup
        tx.send(()).unwrap();
        request1.await?;

        Ok(())
    }

    // Retry mechanism test
    #[tokio::test]
    async fn test_retry_mechanism() -> Result<(), AppError> {
        let config = ReliabilityConfig {
            retry: RetryConfig {
                enabled: true,
                max_attempts: 3,
                base_delay_ms: 10,
                use_exponential_backoff: true,
                retry_status_codes: vec![StatusCode::INTERNAL_SERVER_ERROR],
                ..Default::default()
            },
            ..Default::default()
        };

        let attempt_counter = Arc::new(std::sync::Mutex::new(0));

        let router = apply_reliability(
            Router::new().route(
                "/",
                get({
                    let counter = attempt_counter.clone();
                    move || {
                        let mut count = counter.lock().unwrap();
                        *count += 1;
                        async move {
                            if *count < 3 {
                                Err(AppError::new(ErrorResponse::new(
                                    ErrorType::InternalServerError,
                                    "Temporary failure".to_string(),
                                )))
                            } else {
                                Ok("Success")
                            }
                        }
                    }
                }),
            ),
            &config,
        )?;

        let response = router
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(*attempt_counter.lock().unwrap(), 3);

        Ok(())
    }

    // Add test for configuration validation
    #[tokio::test]
    async fn test_invalid_rate_limit_config() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_window: 0,
            ..Default::default()
        };

        let result = build_rate_limit_layer(&config);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.error.code, "CONFIGURATION_ERROR");
        assert!(error.error.message.contains("requests_per_window"));
    }

    // Add test for configuration safety limits
    #[tokio::test]
    async fn test_safety_limits() {
        let invalid_config = TimeoutConfig {
            enabled: true,
            timeout_seconds: MAX_TIMEOUT_SECONDS + 1,
        };

        let result = build_timeout_layer(&invalid_config);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.error.code, "CONFIGURATION_ERROR");
        assert!(error.error.message.contains("exceeds maximum"));
    }
}

// Update circuit breaker error handling
impl From<circuit_breaker::CircuitBreakerError> for AppError {
    fn from(err: circuit_breaker::CircuitBreakerError) -> Self {
        let error_response = ErrorResponse::new(
            ErrorType::CircuitBreakerOpen,
            "Service temporarily unavailable. Please try again later.".to_string(),
        )
        .with_request_id(get_req_id())
        .with_details(json!({
            "reset_in_seconds": err.reset_in.as_secs(),
            "failure_rate": format!("{:.2}%", err.failure_rate),
            "suggested_retry_after": err.reset_in.as_secs()
        }));

        // Structured logging with failure analysis
        error!(
            error_code = error_response.error.code,
            error_message = error_response.error.message,
            reset_in_seconds = err.reset_in.as_secs(),
            failure_rate = format!("{:.2}%", err.failure_rate),
            window_seconds = err.window_seconds,
            "Circuit breaker activated",
        );

        AppError::new(error_response)
    }
}

// Enhanced error conversion with request context
impl From<rate_limit::RateLimitError> for AppError {
    fn from(err: rate_limit::RateLimitError) -> Self {
        let error_response =
            ErrorResponse::new(ErrorType::TooManyRequests, "Too many requests".to_string())
                .with_request_id(get_req_id())
                .with_details(json!({
                    "retry_after_seconds": err.retry_after.as_secs(),
                    "limit": err.limit
                }));

        warn!(
            error_code = error_response.error.code,
            error_message = error_response.error.message,
            retry_after_seconds = err.retry_after.as_secs(),
            limit = err.limit,
            "Rate limit exceeded"
        );

        AppError::new(error_response)
    }
}

// Add masked error logging for sensitive data
impl From<concurrency::ConcurrencyLimitError> for AppError {
    fn from(err: concurrency::ConcurrencyLimitError) -> Self {
        let error_response = ErrorResponse::new(
            ErrorType::TooManyRequests,
            "Server busy, try again later".to_string(),
        )
        .with_request_id(get_req_id())
        .with_details(json!({
            "current_connections": err.current_connections,
            "max_connections": err.max_connections
        }));

        warn!(
            error_code = error_response.error.code,
            error_message = error_response.error.message,
            current_connections = err.current_connections,
            max_connections = err.max_connections,
            request_id = %error_response.request_id.unwrap_or_default(),
            "Concurrency limit reached"
        );

        AppError::new(error_response)
    }
}

// Add retry error handling
impl From<retry::RetryError> for AppError {
    fn from(err: retry::RetryError) -> Self {
        let error_response = ErrorResponse::new(
            ErrorType::ServiceUnavailable,
            "Service temporarily unavailable".to_string(),
        )
        .with_details(json!({
            "attempts": err.attempts,
            "final_status": err.final_status.as_u16()
        }));

        error!(
            error_code = error_response.error.code,
            error_message = error_response.error.message,
            attempts = err.attempts,
            final_status = err.final_status.as_u16(),
            "Request retries exhausted"
        );

        AppError::new(error_response)
    }
}

// Add safety limits to prevent misconfiguration
const MAX_TIMEOUT_SECONDS: u64 = 300; // 5 minutes
const MAX_CONCURRENT_REQUESTS: u32 = 10_000;
