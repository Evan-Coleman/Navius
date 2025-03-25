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
//! ```rust
//! use axum::Router;
//! use std::time::Duration;
//! use crate::core::reliability::retry::RetryConfig;
//!
//! let retry_config = RetryConfig {
//!     max_attempts: 3,
//!     base_delay: Duration::from_millis(100),
//!     max_delay: Duration::from_secs(1),
//!     use_exponential_backoff: true,
//!     retry_status_codes: vec![
//!         StatusCode::INTERNAL_SERVER_ERROR,
//!         StatusCode::BAD_GATEWAY
//!     ],
//! };
//!
//! let app = Router::new()
//!     .layer(retry_config.layer());
//! ```
//!
//! ## Example with Default Configuration
//!
//! ```rust
//! use crate::core::reliability::retry::RetryConfig;
//!
//! let app = Router::new()
//!     .layer(RetryConfig::default().layer());
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
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::core::config::app_config::RetryConfig as AppConfigRetryConfig;
use crate::core::error::AppError;
use crate::core::error::error_types::Result;
use crate::core::error::{ErrorResponse, ErrorType};
use tower::ServiceBuilder;
use tower::retry::Policy;
use tower::retry::RetryLayer as TowerRetryLayer;
use tower::retry::backoff::ExponentialBackoff as TowerExponentialBackoff;
use tower::util::Either;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};

// Define the Error type that's used in the Policy implementation
type Error = Box<dyn std::error::Error + Send + Sync>;

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
    backoff: TowerExponentialBackoff,
}

impl RetryPolicy {
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            retry_status_codes: vec![500, 502, 503, 504],
            backoff: TowerExponentialBackoff::from_millis(100),
        }
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn with_status_codes(mut self, status_codes: Vec<u16>) -> Self {
        self.retry_status_codes = status_codes;
        self
    }

    pub fn with_backoff(mut self, backoff: TowerExponentialBackoff) -> Self {
        self.backoff = backoff;
        self
    }
}

impl<B> Policy<Request<B>, Response<axum::body::Body>, Error> for RetryPolicy {
    type Future = Either<Ready<Self::RetryAction>, Ready<Self::RetryAction>>;

    fn retry(
        &self,
        req: &Request<B>,
        result: Result<&Response<B>, &Error>,
    ) -> Option<Self::Future> {
        match result {
            Ok(response) => {
                if self
                    .retry_status_codes
                    .contains(&response.status().as_u16())
                {
                    Some(Either::A(ready(RetryAction::Retry)))
                } else {
                    None
                }
            }
            Err(_) => Some(Either::B(ready(RetryAction::Retry))),
        }
    }

    fn clone_request(&self, req: &Request<B>) -> Option<Request<B>> {
        Some(req.try_clone().ok()?)
    }
}

/// Configuration parameters for retry behavior
#[derive(Clone)]
pub struct RetryConfig {
    /// Total maximum attempts (initial request + retries)
    /// Example: 3 = 1 initial + 2 retries
    pub max_attempts: usize,

    /// Base delay between retry attempts. With exponential backoff,
    /// this is multiplied by 2^(attempt number)
    pub base_delay: Duration,

    /// Maximum delay cap for backoff calculations
    pub max_delay: Duration,

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
            backoff: TowerExponentialBackoff::new(self.base_delay)
                .max_delay(self.max_delay)
                .factor(2),
        };

        RetryLayer::new(policy).max_retries(self.max_attempts)
    }

    /// Validate the retry configuration
    pub fn validate(&self) -> Result<(), AppError> {
        if self.max_attempts == 0 {
            return Err(AppError::new(ErrorResponse::new(
                ErrorType::ConfigurationError,
                "max_attempts must be at least 1".to_string(),
            )));
        }

        if self.base_delay > self.max_delay {
            return Err(AppError::new(ErrorResponse::new(
                ErrorType::ConfigurationError,
                format!(
                    "base_delay ({:?}) cannot exceed max_delay ({:?})",
                    self.base_delay, self.max_delay
                ),
            )));
        }

        Ok(())
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            use_exponential_backoff: true,
            retry_status_codes: vec![
                StatusCode::INTERNAL_SERVER_ERROR,
                StatusCode::BAD_GATEWAY,
                StatusCode::SERVICE_UNAVAILABLE,
                StatusCode::GATEWAY_TIMEOUT,
                StatusCode::TOO_MANY_REQUESTS,
            ],
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
        return Err(AppError::ConfigurationError(
            "Retry policy is not enabled".to_string(),
        ));
    }

    let mut policy = RetryPolicy::new()
        .with_max_attempts(config.max_attempts as u32)
        .with_status_codes(
            config
                .retry_status_codes
                .iter()
                .map(|&s| s.as_u16())
                .collect(),
        );

    let backoff = if config.use_exponential_backoff {
        TowerExponentialBackoff::from_millis(config.base_delay)
            .max_delay(Duration::from_millis(config.max_delay))
    } else {
        TowerExponentialBackoff::from_millis(config.base_delay)
            .factor(1.0)
            .max_delay(Duration::from_millis(config.max_delay))
    };

    policy = policy.with_backoff(backoff);

    Ok(policy)
}

impl From<RetryError> for AppError {
    fn from(err: RetryError) -> Self {
        AppError::internal_server_error(format!("Request failed after {} retries", err.attempts))
    }
}
