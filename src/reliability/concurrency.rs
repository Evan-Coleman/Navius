use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Instant;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::future::BoxFuture;
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
    tracker: Arc<Mutex<ConcurrencyTracker>>,
}

impl ConcurrencyLimitLayer {
    /// Create a new concurrency limit layer
    pub fn new(max_concurrent: u32) -> Self {
        Self {
            tracker: Arc::new(Mutex::new(ConcurrencyTracker::new(max_concurrent))),
        }
    }
}

impl<S> Layer<S> for ConcurrencyLimitLayer {
    type Service = ConcurrencyLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ConcurrencyLimitService {
            inner: service,
            tracker: self.tracker.clone(),
        }
    }
}

/// Service implementing concurrency limiting
#[derive(Clone)]
pub struct ConcurrencyLimitService<S> {
    inner: S,
    tracker: Arc<Mutex<ConcurrencyTracker>>,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for ConcurrencyLimitService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = ConcurrencyFuture<S, ReqBody, ResBody>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Check if we're below the concurrency limit
        let mut tracker = self.tracker.lock().unwrap();

        if tracker.try_acquire() {
            debug!(
                "Concurrency permit acquired, current count: {}/{}",
                tracker.count, tracker.max_concurrent
            );

            // Release the permit we just acquired, it will be properly acquired in the call method
            tracker.release();

            // Make sure the inner service is ready
            self.inner.poll_ready(cx).map_err(Into::into)
        } else {
            // Register waker for when a permit becomes available
            debug!(
                "Waiting for concurrency permit, current count: {}/{}",
                tracker.count, tracker.max_concurrent
            );
            tracker.register_waiter(cx.waker());
            Poll::Pending
        }
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Try to acquire a permit
        let permit = {
            let mut tracker = self.tracker.lock().unwrap();

            if !tracker.try_acquire() {
                // This shouldn't happen because poll_ready should have ensured we have capacity,
                // but handle it by returning a 503 response
                warn!(
                    "Concurrency limit exceeded for {} despite poll_ready check",
                    req.uri().path()
                );
                return ConcurrencyFuture {
                    inner: InnerFuture::Rejected,
                    permit: None,
                    tracker: self.tracker.clone(),
                };
            }

            // Create a permit to track this request
            Some(ConcurrencyPermit {
                tracker: self.tracker.clone(),
            })
        };

        // Log concurrency info
        let count = self.tracker.lock().unwrap().count;
        let max = self.tracker.lock().unwrap().max_concurrent;
        debug!("Handling request with concurrency {}/{}", count, max);

        // Create clones for the future
        let service = self.inner.clone();
        let tracker = self.tracker.clone();

        // Create the future to handle the request
        ConcurrencyFuture {
            inner: InnerFuture::Pending {
                service,
                request: req,
                future: None,
            },
            permit,
            tracker,
        }
    }
}

/// Possible states of the concurrency future
enum InnerFuture<S, ReqBody, ResBody> {
    /// Service call is pending
    Pending {
        service: S,
        request: axum::http::Request<ReqBody>,
        future: Option<
            BoxFuture<'static, Result<Response<ResBody>, Box<dyn std::error::Error + Send + Sync>>>,
        >,
    },
    /// Request was rejected due to concurrency limit
    Rejected,
    /// Empty state for Poll
    Empty,
}

/// Future for concurrency limiting service
#[pin_project]
pub struct ConcurrencyFuture<S, ReqBody, ResBody> {
    #[pin]
    inner: InnerFuture<S, ReqBody, ResBody>,
    permit: Option<ConcurrencyPermit>,
    tracker: Arc<Mutex<ConcurrencyTracker>>,
}

impl<S, ReqBody, ResBody> Future for ConcurrencyFuture<S, ReqBody, ResBody>
where
    S: Service<axum::http::Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Output = Result<Response, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner {
            InnerFuture::Pending {
                service,
                request,
                future,
            } => {
                if future.is_none() {
                    // First poll, start the service call
                    let mut svc = service.clone();
                    let req = request.clone();

                    // Create the future
                    *future = Some(Box::pin(async move {
                        let response = svc.call(req).await.map_err(Into::into)?;
                        Ok(response)
                    }));
                }

                // Poll the future
                if let Some(f) = future {
                    return Pin::new(f).poll(cx);
                } else {
                    // This should not happen
                    return Poll::Ready(Err("Concurrency future in invalid state".into()));
                }
            }
            InnerFuture::Rejected => {
                // Service unavailable response
                let response = (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service is at capacity. Please try again later.",
                )
                    .into_response();

                return Poll::Ready(Ok(response));
            }
            InnerFuture::Empty => {
                panic!("ConcurrencyFuture polled after completion");
            }
        }
    }
}
