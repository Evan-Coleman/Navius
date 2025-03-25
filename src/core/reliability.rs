//! Reliability features
//!
//! This module provides middleware components for enhancing application resilience:
//! - Retry mechanisms
//! - Circuit breakers
//! - Rate limiting
//! - Concurrency control
//! - Request timeouts
pub use circuit_breaker::CircuitBreakerError;
pub mod circuit_breaker;
pub mod concurrency;
pub mod metrics;
pub mod rate_limit;
pub mod retry;

use crate::core::reliability::retry::RetryPolicy;

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
pub use circuit_breaker::CircuitBreakerConfig as CbConfig;
pub use concurrency::ConcurrencyLimitLayer;
pub use rate_limit::RateLimitLayer;
pub use retry::RetryConfig as ReliabilityRetryConfig;

use crate::core::reliability::rate_limit::{ConcurrencyLimitError, RateLimitError, RetryError};
use crate::core::reliability::retry::RetryLayer;
use axum::Router;
use axum::body::HttpBody;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tower::util::MapErr;
use tower::{BoxError, Layer, Service, ServiceBuilder, ServiceExt};
use tower_http::timeout::TimeoutLayer;
use tracing::error;
use tracing::{info, warn};

use crate::core::config::app_config::{
    CircuitBreakerConfig, ConcurrencyConfig, RateLimitConfig, ReliabilityConfig, RetryConfig,
    TimeoutConfig,
};
use crate::core::error::AppError;
use crate::core::error::{ErrorResponse, ErrorType};
use crate::core::router::AppState;
use crate::core::utils::request_id;
use crate::core::utils::request_id::get_req_id;
use tower::retry::backoff::ExponentialBackoff;

/// Apply reliability middleware to the router based on configuration
pub fn apply_reliability(
    router: Router,
    config: &ReliabilityConfig,
) -> std::result::Result<Router, AppError> {
    // Create a service builder to chain all the layers
    let mut builder = tower::ServiceBuilder::new();

    // Add timeout layer if enabled
    if config.timeout.enabled {
        if let Some(layer) = build_timeout_layer(&config.timeout)? {
            builder = builder.layer(layer);
        }
    }

    // Add circuit breaker if enabled
    if config.circuit_breaker.enabled {
        if let Some(layer) = build_circuit_breaker_layer(&config.circuit_breaker)? {
            builder = builder.layer(layer);
        }
    }

    // Add rate limiting if enabled
    if config.rate_limit.enabled {
        if let Some(layer) = build_rate_limit_layer(&config.rate_limit)? {
            builder = builder.layer(layer);
        }
    }

    // Add concurrency limiting if enabled
    if config.concurrency.enabled {
        if let Some(layer) = build_concurrency_layer(&config.concurrency)? {
            builder = builder.layer(layer);
        }
    }

    // Return the router with all layers applied
    Ok(router.layer(builder.into_inner()))
}

/// Build the retry layer based on configuration
fn build_retry_layer(config: &RetryConfig) -> Result<Option<RetryLayer>, AppError> {
    if !config.enabled {
        info!("Request retries are disabled");
        return Ok(None);
    }

    // Validate retry configuration
    if config.max_attempts < 1 {
        return Err(AppError::validation_error(
            "Retry max_attempts must be at least 1",
        ));
    }

    if config.base_delay_ms == 0 {
        return Err(AppError::validation_error(
            "Retry base_delay_ms must be greater than 0",
        ));
    }

    info!(
        "Configuring request retries: max_attempts={}, base_delay={}ms, exponential_backoff={}",
        config.max_attempts, config.base_delay_ms, config.use_exponential_backoff
    );

    let retry_policy = RetryPolicy::new(config.max_attempts);
    Ok(Some(RetryLayer::new(retry_policy)))
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
        return Err(AppError::validation_error(
            "Invalid rate limit configuration: requests_per_window cannot be zero",
        ));
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
        return Ok(None);
    }

    let timeout_secs = config.timeout_seconds;
    if timeout_secs == 0 {
        return Err(AppError::validation_error(
            "Timeout must be greater than 0 seconds",
        ));
    }

    let timeout = Duration::from_secs(timeout_secs);
    let layer = TimeoutLayer::new(timeout);
    Ok(Some(layer))
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
        return Err(AppError::validation_error(
            "Concurrency limit must be greater than 0",
        ));
    }

    if config.max_concurrent_requests > MAX_CONCURRENT_REQUESTS {
        return Err(AppError::validation_error(format!(
            "Concurrency limit exceeds maximum allowed value of {}",
            MAX_CONCURRENT_REQUESTS
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
        let mut error_response = ErrorResponse::new(
            "CIRCUIT_BREAKER_OPEN".to_string(),
            "Service temporarily unavailable. Please try again later.".to_string(),
        );
        error_response.details = Some(
            serde_json::to_string(&json!({
                "reset_timeout": err.reset_timeout.as_secs(),
                "failure_rate": err.failure_rate
            }))
            .unwrap(),
        );
        AppError::internal_server_error(error_response.message.clone())
    }
}

// Enhanced error conversion with request context
impl From<rate_limit::RateLimitError> for AppError {
    fn from(err: rate_limit::RateLimitError) -> Self {
        let mut error_response = ErrorResponse::new(
            "TOO_MANY_REQUESTS".to_string(),
            "Too many requests".to_string(),
        );
        error_response.details = Some(
            serde_json::to_string(&json!({
                "retry_after": err.retry_after.as_secs(),
                "rate_limit": err.rate_limit,
                "window_duration": err.window_duration.as_secs()
            }))
            .unwrap(),
        );
        AppError::RateLimited(error_response.message.clone())
    }
}

// Fix ConcurrencyLimitError implementation
impl From<ConcurrencyLimitError> for AppError {
    fn from(err: ConcurrencyLimitError) -> Self {
        AppError::internal_server_error(format!(
            "Concurrency limit of {} exceeded",
            err.max_connections
        ))
    }
}

// Fix RetryError implementation
impl From<RetryError> for AppError {
    fn from(err: RetryError) -> Self {
        AppError::internal_server_error(format!("Request failed after {} retries", err.attempts))
    }
}

// Add safety limits to prevent misconfiguration
const MAX_TIMEOUT_SECONDS: u64 = 300; // 5 minutes
const MAX_CONCURRENT_REQUESTS: u32 = 10_000;
