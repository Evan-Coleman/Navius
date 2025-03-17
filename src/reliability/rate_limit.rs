use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use futures::future::BoxFuture;
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{debug, info, warn};

/// Token bucket rate limiter implementation
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Maximum number of tokens the bucket can hold
    capacity: u32,
    /// Current number of tokens in the bucket
    tokens: u32,
    /// Time between token refills
    refill_interval: Duration,
    /// Last time the bucket was refilled
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(capacity: u32, refill_interval: Duration) -> Self {
        Self {
            capacity,
            tokens: capacity, // Start with a full bucket
            refill_interval,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume a token from the bucket
    fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Calculate how many tokens to add based on elapsed time
        if elapsed >= self.refill_interval {
            let periods = (elapsed.as_millis() as u64) / self.refill_interval.as_millis() as u64;

            if periods > 0 {
                let tokens_to_add = (periods as u32).saturating_mul(1);
                self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
                self.last_refill = now;
            }
        }
    }
}

/// Store for rate limit buckets
#[derive(Debug, Clone)]
struct RateLimitStore<K> {
    /// Map of client keys to token buckets
    buckets: Arc<Mutex<HashMap<K, TokenBucket>>>,
    /// Default bucket capacity
    capacity: u32,
    /// Default refill interval
    refill_interval: Duration,
}

impl<K: Eq + Hash + Clone> RateLimitStore<K> {
    /// Create a new rate limit store
    fn new(capacity: u32, window: Duration) -> Self {
        // Calculate refill interval based on capacity and window
        let refill_interval = window.div_f64(capacity as f64);

        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            refill_interval,
        }
    }

    /// Try to consume a token for the given key
    fn try_consume(&self, key: &K) -> bool {
        let mut buckets = self.buckets.lock().unwrap();

        // Get or create a bucket for this key
        let bucket = buckets
            .entry(key.clone())
            .or_insert_with(|| TokenBucket::new(self.capacity, self.refill_interval));

        bucket.try_consume()
    }
}

/// Global rate limit store (shared across all clients)
struct GlobalRateLimiter {
    /// Token bucket for all requests
    bucket: Arc<Mutex<TokenBucket>>,
}

impl GlobalRateLimiter {
    /// Create a new global rate limiter
    fn new(capacity: u32, window: Duration) -> Self {
        // Calculate refill interval based on capacity and window
        let refill_interval = window.div_f64(capacity as f64);

        Self {
            bucket: Arc::new(Mutex::new(TokenBucket::new(capacity, refill_interval))),
        }
    }

    /// Try to consume a token
    fn try_consume(&self) -> bool {
        let mut bucket = self.bucket.lock().unwrap();
        bucket.try_consume()
    }
}

/// Layer for adding rate limiting capability to services
#[derive(Clone)]
pub struct RateLimitLayer {
    /// Global rate limiter (applied to all requests)
    global_limiter: Arc<GlobalRateLimiter>,
    /// Per-client rate limiter (if enabled)
    client_limiter: Option<Arc<RateLimitStore<IpAddr>>>,
}

impl RateLimitLayer {
    /// Create a new rate limit layer
    pub fn new(requests_per_window: u32, window: Duration, per_client: bool) -> Self {
        // Create global rate limiter
        let global_limiter = Arc::new(GlobalRateLimiter::new(requests_per_window, window));

        // Create per-client rate limiter if enabled
        let client_limiter = if per_client {
            Some(Arc::new(RateLimitStore::new(requests_per_window, window)))
        } else {
            None
        };

        Self {
            global_limiter,
            client_limiter,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimitService {
            inner: service,
            global_limiter: self.global_limiter.clone(),
            client_limiter: self.client_limiter.clone(),
        }
    }
}

/// Service implementing rate limiting
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    global_limiter: Arc<GlobalRateLimiter>,
    client_limiter: Option<Arc<RateLimitStore<IpAddr>>>,
}

/// Rate limit exceeded error response
fn rate_limit_exceeded() -> Response {
    (
        axum::http::StatusCode::TOO_MANY_REQUESTS,
        "Rate limit exceeded. Please try again later.",
    )
        .into_response()
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = RateLimitFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Apply global rate limit
        if !self.global_limiter.try_consume() {
            warn!("Global rate limit exceeded for {}", req.uri().path());
            return RateLimitFuture {
                inner: Box::pin(async { Ok(rate_limit_exceeded()) }),
            };
        }

        // Apply per-client rate limit if enabled
        if let Some(client_limiter) = &self.client_limiter {
            // Try to get client IP from ConnectInfo extension
            if let Some(client_ip) = req
                .extensions()
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|connect_info| connect_info.0.ip())
            {
                if !client_limiter.try_consume(&client_ip) {
                    warn!("Client rate limit exceeded for IP: {}", client_ip);
                    return RateLimitFuture {
                        inner: Box::pin(async { Ok(rate_limit_exceeded()) }),
                    };
                }

                debug!("Rate limit check passed for client: {}", client_ip);
            } else {
                debug!("Could not get client IP for rate limiting");
            }
        }

        // Rate limit checks passed, call the inner service
        let future = self.inner.call(req).map_err(Into::into);

        RateLimitFuture {
            inner: Box::pin(future),
        }
    }
}

/// Future for rate limiting service
#[pin_project]
pub struct RateLimitFuture<F> {
    #[pin]
    inner: BoxFuture<'static, Result<Response, Box<dyn std::error::Error + Send + Sync>>>,
}

impl<F> Future for RateLimitFuture<F> {
    type Output = Result<Response, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}
