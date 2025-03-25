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
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use tower::{Layer, Service};
use tracing::{debug, info, warn};

use rand::SeedableRng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::core::error::AppError;
use tower::ServiceBuilder;
use tower::retry::{ExponentialBackoff, RetryLayer};

/// Type alias for our configured retry layer using Tower's RetryLayer
/// with a custom [RetryPolicy] implementation.
pub type RetryLayer = tower::retry::RetryLayer<RetryPolicy>;

/// Custom retry policy determining which requests should be retried
/// based on HTTP status codes and service errors.
#[derive(Clone)]
pub struct RetryPolicy {
    /// HTTP status codes that should trigger automatic retries
    /// Example: 500, 502, 503, 504
    status_codes: Vec<StatusCode>,
}

impl<E> tower::retry::Policy<Request<axum::body::Body>, Response<axum::body::Body>, E>
    for RetryPolicy
where
    E: std::fmt::Debug,
{
    type Future = futures::future::Ready<Self>;

    fn retry(
        &self,
        req: &Request<axum::body::Body>,
        result: Result<&Response<axum::body::Body>, &E>,
    ) -> Option<Self::Future> {
        match result {
            // Retry on configured status codes
            Ok(response) if self.status_codes.contains(&response.status()) => {
                Some(futures::future::ready(self.clone()))
            }
            // Retry on any service errors
            Err(_) => Some(futures::future::ready(self.clone())),
            // Don't retry successful responses
            _ => None,
        }
    }

    fn clone_request(&self, req: &Request<axum::body::Body>) -> Option<Request<axum::body::Body>> {
        Some(req.clone())
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
            status_codes: self.retry_status_codes.clone(),
        };

        // Configure backoff strategy
        let backoff = if self.use_exponential_backoff {
            ExponentialBackoff::new(self.base_delay)
                .max_delay(self.max_delay)
                .factor(2) // Double delay each attempt
        } else {
            ExponentialBackoff::constant(self.base_delay).max_delay(self.max_delay)
        };

        // Build the retry layer
        RetryLayer::new(policy)
            .with_backoff(backoff)
            .max_retries(self.max_attempts)
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
