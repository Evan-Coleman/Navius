//! # Retry Middleware
//!
//! Configurable retry functionality for HTTP requests with:
//! - Exponential backoff with jitter
//! - Status code-based retry triggers
//! - Configurable attempt limits
//! - Tower middleware integration
//!
//! ## Example
//!
//! ```
//! use axum::http::StatusCode;
//! use navius::core::reliability::retry::RetryConfig;
//!
//! let retry_config = RetryConfig {
//!     max_attempts: 3,
//!     base_delay: 100, // milliseconds
//!     max_delay: 1000, // milliseconds
//!     use_exponential_backoff: true,
//!     retry_status_codes: vec![
//!         StatusCode::INTERNAL_SERVER_ERROR,
//!         StatusCode::BAD_GATEWAY
//!     ],
//!     enabled: true,
//! };
//! ```
//!
//! ## Example with Default Configuration
//!
//! ```
//! use navius::core::reliability::retry::RetryConfig;
//!
//! // Create a default retry configuration
//! let retry_config = RetryConfig::default();
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use axum::http::Request;
use axum::http::StatusCode;
use axum::response::Response;
use futures::future::{self, BoxFuture, Ready};
use futures::{FutureExt, TryFutureExt};
use tower::{Layer, Service};
use tracing::{debug, info, warn};

use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::core::config::app_config::RetryConfig as AppConfigRetryConfig;
use crate::core::error::AppError;
use crate::core::error::error_types::Result;
use crate::core::error::{ErrorResponse, ErrorType};
use tower::ServiceBuilder;
use tower::retry::Policy;
use tower::retry::RetryLayer as TowerRetryLayer;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};

// Define the Error type that's used in the Policy implementation
type Error = Box<dyn std::error::Error + Send + Sync>;

// RetryAction enum for the Policy implementation
pub enum RetryAction {
    Retry,
    Return,
}

// Helper for creating Ready futures
fn ready<T>(value: T) -> Ready<T> {
    future::ready(value)
}

/// Type alias for our configured retry layer using Tower's RetryLayer
/// with a custom [RetryPolicy] implementation.
pub type RetryLayer = tower::retry::RetryLayer<RetryPolicy>;

/// Custom retry policy determining which requests should be retried
/// based on HTTP status codes and service errors.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: u32,
    retry_status_codes: Vec<u16>,
    current_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            retry_status_codes: vec![500, 502, 503, 504],
            current_attempts: 0,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
        }
    }

    pub fn with_status_codes(mut self, status_codes: Vec<u16>) -> Self {
        self.retry_status_codes = status_codes;
        self
    }

    pub fn with_backoff(mut self, base_delay: Duration, max_delay: Duration) -> Self {
        self.base_delay = base_delay;
        self.max_delay = max_delay;
        self
    }

    // Calculate backoff delay using exponential backoff with jitter
    fn get_backoff_duration(&self) -> Duration {
        let base = self.base_delay.as_millis() as f64;
        let attempt = self.current_attempts as f64;

        // Exponential backoff: base * 2^attempt
        let exp_backoff = base * 2.0_f64.powf(attempt);

        // Apply jitter (0.5-1.5 of the calculated value)
        let jitter = 0.5 + rand::random::<f64>();
        let with_jitter = exp_backoff * jitter;

        // Cap at max_delay
        let capped = with_jitter.min(self.max_delay.as_millis() as f64);

        Duration::from_millis(capped as u64)
    }
}

impl<B> Policy<Request<B>, Response<axum::body::Body>, Error> for RetryPolicy
where
    B: Clone + Send + 'static,
{
    type Future = Ready<()>;

    fn retry(
        &mut self,
        _req: &mut Request<B>,
        result: &mut std::result::Result<Response<axum::body::Body>, Error>,
    ) -> Option<Self::Future> {
        self.current_attempts += 1;

        if self.current_attempts >= self.max_attempts {
            return None;
        }

        let should_retry = match result {
            Ok(response) => self
                .retry_status_codes
                .contains(&response.status().as_u16()),
            Err(_) => true,
        };

        if should_retry { Some(ready(())) } else { None }
    }

    fn clone_request(&mut self, req: &Request<B>) -> Option<Request<B>> {
        Some(req.clone())
    }
}

/// Configuration parameters for retry behavior
#[derive(Clone)]
pub struct RetryConfig {
    /// Whether retries are enabled
    pub enabled: bool,

    /// Total maximum attempts (initial request + retries)
    /// Example: 3 = 1 initial + 2 retries
    pub max_attempts: usize,

    /// Base delay between retry attempts. With exponential backoff,
    /// this is multiplied by 2^(attempt number)
    pub base_delay: u64,

    /// Maximum delay cap for backoff calculations
    pub max_delay: u64,

    /// Use exponential backoff (true) or fixed delays (false)
    pub use_exponential_backoff: bool,

    /// HTTP status codes that should trigger automatic retries
    /// Default: 500, 502, 503, 504, 429
    pub retry_status_codes: Vec<StatusCode>,
}

impl RetryConfig {
    /// Create a Tower layer with the configured retry policy
    pub fn layer(&self) -> RetryLayer {
        let policy = RetryPolicy {
            max_attempts: self.max_attempts as u32,
            retry_status_codes: self
                .retry_status_codes
                .iter()
                .map(|&s| s.as_u16())
                .collect(),
            current_attempts: 0,
            base_delay: Duration::from_millis(self.base_delay),
            max_delay: Duration::from_millis(self.max_delay),
        };

        RetryLayer::new(policy)
    }

    /// Validate the retry configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_attempts == 0 {
            return Err(AppError::validation_error(
                "max_attempts must be at least 1",
            ));
        }

        if self.base_delay > self.max_delay {
            return Err(AppError::validation_error(format!(
                "base_delay ({}) cannot exceed max_delay ({})",
                self.base_delay, self.max_delay
            )));
        }

        Ok(())
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: 100,
            max_delay: 1000,
            use_exponential_backoff: true,
            retry_status_codes: vec![
                StatusCode::INTERNAL_SERVER_ERROR,
                StatusCode::BAD_GATEWAY,
                StatusCode::SERVICE_UNAVAILABLE,
                StatusCode::GATEWAY_TIMEOUT,
                StatusCode::TOO_MANY_REQUESTS,
            ],
            enabled: true,
        }
    }
}

#[derive(Debug)]
pub struct RetryError {
    pub attempts: u32,
}

impl std::fmt::Display for RetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Request failed after {} retry attempts", self.attempts)
    }
}

impl std::error::Error for RetryError {}

pub fn build_retry_policy(config: &RetryConfig) -> Result<RetryPolicy> {
    if !config.enabled {
        return Err(AppError::validation_error("Retry policy is not enabled"));
    }

    let policy = RetryPolicy::new(config.max_attempts as u32)
        .with_status_codes(
            config
                .retry_status_codes
                .iter()
                .map(|&s| s.as_u16())
                .collect(),
        )
        .with_backoff(
            Duration::from_millis(config.base_delay),
            Duration::from_millis(config.max_delay),
        );

    Ok(policy)
}

impl From<RetryError> for AppError {
    fn from(err: RetryError) -> Self {
        AppError::internal_server_error(format!("Request failed after {} retries", err.attempts))
    }
}
