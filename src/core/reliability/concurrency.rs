use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Instant;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::{FutureExt, TryFutureExt, future::BoxFuture};
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{debug, info, warn};

/// Concurrency tracker
#[derive(Debug)]
struct ConcurrencyTracker {
    /// Current count of in-flight requests
    count: u32,
    /// Maximum allowed concurrent requests
    max_concurrent: u32,
    /// Wakers for tasks waiting for capacity
    waiters: Vec<Waker>,
    /// Start times for in-flight requests (for debugging)
    start_times: Vec<Instant>,
}

impl ConcurrencyTracker {
    /// Create a new concurrency tracker
    fn new(max_concurrent: u32) -> Self {
        Self {
            count: 0,
            max_concurrent,
            waiters: Vec::new(),
            start_times: Vec::new(),
        }
    }

    /// Try to acquire a concurrency permit
    fn try_acquire(&mut self) -> bool {
        if self.count < self.max_concurrent {
            self.count += 1;
            self.start_times.push(Instant::now());
            true
        } else {
            false
        }
    }

    /// Release a concurrency permit and wake waiters
    fn release(&mut self) {
        if self.count > 0 {
            self.count -= 1;
            if !self.start_times.is_empty() {
                self.start_times.remove(0);
            }

            // Wake one waiter if available
            if !self.waiters.is_empty() {
                let waker = self.waiters.remove(0);
                waker.wake();
            }
        }
    }

    /// Register a waker for when capacity becomes available
    fn register_waiter(&mut self, waker: &Waker) {
        // Check if this waker is already in our list
        if !self.waiters.iter().any(|w| w.will_wake(waker)) {
            self.waiters.push(waker.clone());
        }
    }
}

/// Permit for tracking concurrency
struct ConcurrencyPermit {
    tracker: Arc<Mutex<ConcurrencyTracker>>,
}

impl Drop for ConcurrencyPermit {
    fn drop(&mut self) {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.release();
    }
}

/// Layer for adding concurrency limiting capability to services
#[derive(Clone)]
pub struct ConcurrencyLimitLayer {
    max_concurrent: u32,
}

impl ConcurrencyLimitLayer {
    /// Create a new concurrency limit layer
    pub fn new(max_concurrent: u32) -> Self {
        Self { max_concurrent }
    }
}

impl<S> Layer<S> for ConcurrencyLimitLayer {
    type Service = ConcurrencyLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ConcurrencyLimitService {
            inner: service,
            tracker: Arc::new(Mutex::new(ConcurrencyTracker::new(self.max_concurrent))),
        }
    }
}

/// Service implementing concurrency limiting
#[derive(Clone)]
pub struct ConcurrencyLimitService<S> {
    inner: S,
    tracker: Arc<Mutex<ConcurrencyTracker>>,
}

/// Type alias for the future response type to reduce complexity
type FutureResponse<ResBody> =
    BoxFuture<'static, Result<Response<ResBody>, Box<dyn std::error::Error + Send + Sync>>>;

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for ConcurrencyLimitService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut state = self.tracker.lock().unwrap();
        if !state.try_acquire() {
            debug!(
                "Waiting for concurrency permit, current count: {}/{}",
                state.count, state.max_concurrent
            );
            state.register_waiter(cx.waker());
            return Poll::Pending;
        }

        debug!(
            "Concurrency permit acquired, current count: {}/{}",
            state.count, state.max_concurrent
        );

        // Release the permit we just acquired, it will be properly acquired in the call method
        state.release();

        // Make sure the inner service is ready
        self.inner.poll_ready(cx).map_err(|e| Box::new(e) as _)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Try to acquire a permit
        let mut state = self.tracker.lock().unwrap();
        if !state.try_acquire() {
            debug!("Concurrency limit reached, rejecting request");
            // Create a response using axum's response builder
            // We'll use a type conversion to handle the body type
            // Unused variable is intentional here, we're just creating the error
            let _response = Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header("Retry-After", "5")
                .body(axum::body::Body::from(
                    "Server is at maximum capacity. Please try again later.",
                ))
                .unwrap();

            // Convert the response to the expected type using a boxed future
            return futures::future::ready(Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Service is at capacity",
            ))
                as Box<dyn std::error::Error + Send + Sync>))
            .boxed();
        }
        drop(state);

        // Get a clone to use after we finish
        let tracker = self.tracker.clone();

        // Clone the service for use in the future
        let clone_service = self.inner.clone();
        let mut service = std::mem::replace(&mut self.inner, clone_service);

        // Call the inner service
        let future = service.call(req);

        async move {
            // Create a guard that will release the permit when dropped
            let _guard = ConcurrencyGuard {
                tracker: tracker.clone(),
            };

            // Call the inner service and propagate the result
            let result = future
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

            Ok(result)
        }
        .boxed()
    }
}

/// Guard to ensure the permit is released when done
struct ConcurrencyGuard {
    tracker: Arc<Mutex<ConcurrencyTracker>>,
}

impl Drop for ConcurrencyGuard {
    fn drop(&mut self) {
        let mut tracker = self.tracker.lock().unwrap();
        tracker.release();
    }
}

/// Possible states of the concurrency future
enum InnerFuture<S, ReqBody, ResBody> {
    /// Service call is pending
    Pending {
        service: S,
        request: axum::http::Request<ReqBody>,
        future: Option<FutureResponse<ResBody>>,
    },
    /// Request was rejected due to concurrency limit
    Rejected,
    /// Empty state for Poll
    Empty,
}

/// Future for concurrency limiting service
#[pin_project]
pub struct ConcurrencyLimitFuture<F> {
    #[pin]
    inner: F,
    state: Arc<Mutex<ConcurrencyTracker>>,
}

impl<F, T, E> Future for ConcurrencyLimitFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    type Output = Result<T, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(response)) => {
                let mut state = this.state.lock().unwrap();
                state.release();
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(error)) => {
                let mut state = this.state.lock().unwrap();
                state.release();
                Poll::Ready(Err(Box::new(error)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
