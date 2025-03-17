//! Reliability middleware for enhancing application resilience
//!
//! This module provides middleware components for:
//! - Retries - Automatically retry failed requests
//! - Circuit Breaker - Prevent cascading failures
//! - Rate Limiting - Control request rates
//! - Concurrency Limiting - Control concurrent request counts
//! - Request Timeouts - Ensure requests complete in a timely manner

mod circuit_breaker;
mod concurrency;
mod metrics;
mod rate_limit;
mod retry;

pub use circuit_breaker::*;
pub use concurrency::*;
pub use metrics::*;
pub use rate_limit::*;
pub use retry::*;

use axum::{Router, response::Response};
use std::sync::Arc;
use tower::{Layer, ServiceBuilder};
use tracing::{info, warn};

use crate::{
    app::AppState,
    config::{
        CircuitBreakerConfig, ConcurrencyConfig, RateLimitConfig, ReliabilityConfig, RetryConfig,
        TimeoutConfig,
    },
};

/// Apply reliability middleware to the router based on configuration
pub fn apply_reliability(router: Router, state: Arc<AppState>) -> Router {
    let config = &state.config.reliability;

    info!("Configuring reliability middleware...");

    // Build the reliability middleware stack
    let mut builder = ServiceBuilder::new();

    // Add timeout layer if enabled
    if let Some(timeout_layer) = build_timeout_layer(&config.timeout) {
        builder = builder.layer(timeout_layer);
    }

    // Add rate limiting if enabled
    if let Some(rate_limit_layer) = build_rate_limit_layer(&config.rate_limit) {
        builder = builder.layer(rate_limit_layer);
    }

    // Add concurrency limits if enabled
    if let Some(concurrency_layer) = build_concurrency_layer(&config.concurrency) {
        builder = builder.layer(concurrency_layer);
    }

    // Add circuit breaker if enabled
    if let Some(circuit_breaker_layer) = build_circuit_breaker_layer(&config.circuit_breaker) {
        builder = builder.layer(circuit_breaker_layer);
    }

    // Add retry layer if enabled
    if let Some(retry_layer) = build_retry_layer(&config.retry) {
        builder = builder.layer(retry_layer);
    }

    // Add reliability metrics layer
    builder = builder.layer(ReliabilityMetricsLayer::new());

    router.layer(builder)
}

/// Build the retry layer based on configuration
fn build_retry_layer(config: &RetryConfig) -> Option<RetryLayer> {
    if !config.enabled {
        info!("Request retries are disabled");
        return None;
    }

    info!(
        "Configuring request retries: max_attempts={}, base_delay={}ms, exponential_backoff={}",
        config.max_attempts, config.base_delay_ms, config.use_exponential_backoff
    );

    Some(RetryLayer::new(
        config.max_attempts,
        Duration::from_millis(config.base_delay_ms),
        Duration::from_millis(config.max_delay_ms),
        config.use_exponential_backoff,
        config.retry_status_codes.clone(),
    ))
}

/// Build the circuit breaker layer based on configuration
fn build_circuit_breaker_layer(config: &CircuitBreakerConfig) -> Option<CircuitBreakerLayer> {
    if !config.enabled {
        info!("Circuit breaker is disabled");
        return None;
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

    Some(CircuitBreakerLayer::new_with_config(
        config.failure_threshold,
        Duration::from_millis(config.reset_timeout_ms),
        config.success_threshold,
        config.window_seconds,
        config.failure_percentage,
        config.use_consecutive_failures,
        config.failure_status_codes.clone(),
    ))
}

/// Build the rate limiting layer based on configuration
fn build_rate_limit_layer(config: &RateLimitConfig) -> Option<RateLimitLayer> {
    if !config.enabled {
        info!("Rate limiting is disabled");
        return None;
    }

    info!(
        "Configuring rate limiter: {} requests per {} seconds, per_client={}",
        config.requests_per_window, config.window_seconds, config.per_client
    );

    Some(RateLimitLayer::new(
        config.requests_per_window,
        Duration::from_secs(config.window_seconds),
        config.per_client,
    ))
}

/// Build the timeout layer based on configuration
fn build_timeout_layer(config: &TimeoutConfig) -> Option<tower_http::timeout::TimeoutLayer> {
    if !config.enabled {
        info!("Request timeouts are disabled");
        return None;
    }

    info!("Configuring request timeouts: {}s", config.timeout_seconds);

    Some(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(
        config.timeout_seconds,
    )))
}

/// Build the concurrency layer based on configuration
fn build_concurrency_layer(config: &ConcurrencyConfig) -> Option<ConcurrencyLimitLayer> {
    if !config.enabled {
        info!("Concurrency limits are disabled");
        return None;
    }

    info!(
        "Configuring concurrency limits: max={}",
        config.max_concurrent_requests
    );

    Some(ConcurrencyLimitLayer::new(config.max_concurrent_requests))
}

use std::time::Duration;
