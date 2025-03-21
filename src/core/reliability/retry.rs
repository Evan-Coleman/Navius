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

/// Layer for adding retry capability to services
#[derive(Clone, Debug)]
pub struct RetryLayer {
    /// Maximum number of retry attempts
    max_attempts: u32,
    /// Base delay between retries
    base_delay: Duration,
    /// Maximum delay between retries
    max_delay: Duration,
    /// Whether to use exponential backoff
    use_exponential_backoff: bool,
    /// Status codes that should trigger a retry
    retry_status_codes: Vec<u16>,
}

impl RetryLayer {
    /// Create a new retry layer
    pub fn new(
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        use_exponential_backoff: bool,
        retry_status_codes: Vec<u16>,
    ) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            use_exponential_backoff,
            retry_status_codes,
        }
    }
}

impl<S> Layer<S> for RetryLayer {
    type Service = RetryService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RetryService {
            inner: service,
            max_attempts: self.max_attempts,
            base_delay: self.base_delay,
            max_delay: self.max_delay,
            use_exponential_backoff: self.use_exponential_backoff,
            retry_status_codes: self.retry_status_codes.clone(),
        }
    }
}

/// Service implementing retry pattern
#[derive(Clone)]
pub struct RetryService<S> {
    inner: S,
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    use_exponential_backoff: bool,
    retry_status_codes: Vec<u16>,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for RetryService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static + Clone,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| Box::new(e) as _)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Clone the service and request to allow for retries
        let clone_service = self.inner.clone();
        let service = std::mem::replace(&mut self.inner, clone_service);

        // Convert status codes to StatusCode objects
        let retry_status_codes: Vec<StatusCode> = self
            .retry_status_codes
            .iter()
            .filter_map(|&code| StatusCode::from_u16(code).ok())
            .collect();

        let max_attempts = self.max_attempts;
        let base_delay = self.base_delay;
        let max_delay = self.max_delay;
        let use_exponential_backoff = self.use_exponential_backoff;

        // Create a thread-safe RNG with a random seed
        let rng_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_nanos() as u64;

        async move {
            let mut attempt = 0;
            let mut service = service;
            let request = req;
            let mut rng = ChaCha8Rng::seed_from_u64(rng_seed);

            loop {
                attempt += 1;
                debug!("Attempt {} of {}", attempt, max_attempts);

                // Clone the request for this attempt
                let cloned_req = clone_request(&request);

                // Call the service
                let response = service
                    .call(cloned_req)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                // Check if we need to retry
                let status = response.status();
                let should_retry = attempt < max_attempts && retry_status_codes.contains(&status);

                if !should_retry {
                    return Ok(response);
                }

                debug!("Retrying request due to status: {}", status);

                // Calculate delay with jitter
                let mut delay = if use_exponential_backoff {
                    let exp_backoff = base_delay.as_millis() as u64 * 2u64.pow(attempt - 1);
                    Duration::from_millis(exp_backoff.min(max_delay.as_millis() as u64))
                } else {
                    base_delay
                };

                // Add jitter (±20%)
                let jitter_factor = 0.8 + (rng.random::<f64>() * 0.4); // 0.8 to 1.2
                let jittered_millis = (delay.as_millis() as f64 * jitter_factor) as u64;
                delay = Duration::from_millis(jittered_millis);

                debug!("Waiting for {:?} before retry", delay);
                tokio::time::sleep(delay).await;

                // Clone the service for the next attempt
                let next_service = service.clone();
                service = next_service;
            }
        }
        .boxed()
    }
}

// Helper function to clone a request
fn clone_request<B: Clone>(req: &Request<B>) -> Request<B> {
    let (parts, body) = req.clone().into_parts();
    Request::from_parts(parts, body)
}
