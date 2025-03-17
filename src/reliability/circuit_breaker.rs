use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use futures::FutureExt;
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{debug, error, info, warn};

/// Circuit state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed - requests flow normally
    Closed,
    /// Circuit is open - requests are rejected without calling the service
    Open,
    /// Circuit is in half-open state - allowing test requests to check if service has recovered
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "CLOSED"),
            CircuitState::Open => write!(f, "OPEN"),
            CircuitState::HalfOpen => write!(f, "HALF-OPEN"),
        }
    }
}

/// Record of a single request result for tracking in the rolling window
#[derive(Debug, Clone)]
struct RequestResult {
    /// Time when the request completed
    timestamp: Instant,
    /// Whether the request was a success or failure
    success: bool,
}

/// Shared circuit breaker state
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    /// Current state of the circuit
    state: CircuitState,

    /// Legacy consecutive failures mode
    /// Number of consecutive failures in the current window
    failure_count: u32,
    /// Number of consecutive successes in half-open state
    success_count: u32,
    /// Number of consecutive failures needed to trip the circuit
    failure_threshold: u32,

    /// Rolling window mode
    /// History of request results within the rolling window
    request_history: VecDeque<RequestResult>,
    /// Time window for tracking failure rate
    window_duration: Duration,
    /// Failure percentage threshold (0-100) that triggers the circuit breaker
    failure_percentage: u8,
    /// Whether to use the legacy consecutive failures mode (false = use rolling window)
    use_consecutive_failures: bool,
    /// HTTP status codes that should be considered failures
    failure_status_codes: Vec<u16>,

    /// Common settings
    /// Number of consecutive successes needed to close the circuit
    success_threshold: u32,
    /// Time to wait before transitioning from open to half-open
    reset_timeout: Duration,
    /// Time when the circuit was opened
    opened_at: Option<Instant>,
}

impl CircuitBreakerState {
    /// Create a new circuit breaker state
    fn new(
        failure_threshold: u32,
        reset_timeout: Duration,
        success_threshold: u32,
        window_seconds: u64,
        failure_percentage: u8,
        use_consecutive_failures: bool,
        failure_status_codes: Vec<u16>,
    ) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            reset_timeout,
            opened_at: None,
            request_history: VecDeque::new(),
            window_duration: Duration::from_secs(window_seconds),
            failure_percentage,
            use_consecutive_failures,
            failure_status_codes,
        }
    }

    /// Prune request history to only include entries within the window
    fn prune_history(&mut self) {
        let now = Instant::now();
        let window_start = now - self.window_duration;

        // Remove entries older than the window
        while let Some(entry) = self.request_history.front() {
            if entry.timestamp < window_start {
                self.request_history.pop_front();
            } else {
                break;
            }
        }
    }

    /// Calculate the current failure percentage in the rolling window
    fn calculate_failure_percentage(&self) -> f32 {
        let total_requests = self.request_history.len();
        if total_requests == 0 {
            return 0.0;
        }

        let failures = self.request_history.iter().filter(|r| !r.success).count();
        (failures as f32 / total_requests as f32) * 100.0
    }

    /// Record a successful request
    fn record_success(&mut self) {
        if self.use_consecutive_failures {
            self.record_success_legacy();
        } else {
            // Add this success to the history
            self.request_history.push_back(RequestResult {
                timestamp: Instant::now(),
                success: true,
            });

            // Prune old entries
            self.prune_history();

            match self.state {
                CircuitState::Closed => {
                    // Nothing to do in closed state for success
                }
                CircuitState::HalfOpen => {
                    // In half-open state, increment success count
                    self.success_count += 1;

                    // If we've reached the success threshold, close the circuit
                    if self.success_count >= self.success_threshold {
                        info!(
                            "Circuit breaker state transition: {} -> CLOSED (success threshold reached)",
                            self.state
                        );
                        self.state = CircuitState::Closed;
                        self.success_count = 0;
                    }
                }
                CircuitState::Open => {
                    // Shouldn't happen, but handle anyway by checking if we should transition to half-open
                    if let Some(opened_at) = self.opened_at {
                        if opened_at.elapsed() >= self.reset_timeout {
                            info!(
                                "Circuit breaker state transition: {} -> HALF-OPEN (reset timeout elapsed)",
                                self.state
                            );
                            self.state = CircuitState::HalfOpen;
                            self.success_count = 1; // Count this success
                        }
                    }
                }
            }
        }
    }

    /// Legacy implementation of record_success
    fn record_success_legacy(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                // In half-open state, increment success count
                self.success_count += 1;

                // If we've reached the success threshold, close the circuit
                if self.success_count >= self.success_threshold {
                    info!(
                        "Circuit breaker state transition: {} -> CLOSED (success threshold reached)",
                        self.state
                    );
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle anyway by checking if we should transition to half-open
                if let Some(opened_at) = self.opened_at {
                    if opened_at.elapsed() >= self.reset_timeout {
                        info!(
                            "Circuit breaker state transition: {} -> HALF-OPEN (reset timeout elapsed)",
                            self.state
                        );
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 1; // Count this success
                    }
                }
            }
        }
    }

    /// Record a failed request
    fn record_failure(&mut self) {
        if self.use_consecutive_failures {
            self.record_failure_legacy();
        } else {
            // Add this failure to the history
            self.request_history.push_back(RequestResult {
                timestamp: Instant::now(),
                success: false,
            });

            // Prune old entries
            self.prune_history();

            match self.state {
                CircuitState::Closed => {
                    // Calculate the current failure percentage
                    let failure_percentage = self.calculate_failure_percentage();

                    // Check if we've exceeded the threshold
                    if failure_percentage >= self.failure_percentage as f32 {
                        warn!(
                            "Circuit breaker state transition: {} -> OPEN (failure percentage {:.1}% exceeds threshold {}%)",
                            self.state, failure_percentage, self.failure_percentage
                        );
                        self.state = CircuitState::Open;
                        self.opened_at = Some(Instant::now());
                    }
                }
                CircuitState::HalfOpen => {
                    // In half-open state, any failure opens the circuit again
                    warn!(
                        "Circuit breaker state transition: {} -> OPEN (failure in half-open state)",
                        self.state
                    );
                    self.state = CircuitState::Open;
                    self.success_count = 0;
                    self.opened_at = Some(Instant::now());
                }
                CircuitState::Open => {
                    // Already open, reset the opened_at time
                    self.opened_at = Some(Instant::now());
                }
            }
        }
    }

    /// Legacy implementation of record_failure
    fn record_failure_legacy(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Increment failure count
                self.failure_count += 1;

                // If we've reached the failure threshold, open the circuit
                if self.failure_count >= self.failure_threshold {
                    warn!(
                        "Circuit breaker state transition: {} -> OPEN (failure threshold reached: {})",
                        self.state, self.failure_threshold
                    );
                    self.state = CircuitState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                // In half-open state, any failure opens the circuit again
                warn!(
                    "Circuit breaker state transition: {} -> OPEN (failure in half-open state)",
                    self.state
                );
                self.state = CircuitState::Open;
                self.success_count = 0;
                self.opened_at = Some(Instant::now());
            }
            CircuitState::Open => {
                // Already open, reset the opened_at time
                self.opened_at = Some(Instant::now());
            }
        }
    }

    /// Check if circuit is open and should transition to half-open
    fn check_transition_to_half_open(&mut self) {
        if self.state == CircuitState::Open {
            if let Some(opened_at) = self.opened_at {
                if opened_at.elapsed() >= self.reset_timeout {
                    info!(
                        "Circuit breaker state transition: {} -> HALF-OPEN (reset timeout elapsed)",
                        self.state
                    );
                    self.state = CircuitState::HalfOpen;
                    self.success_count = 0;
                }
            }
        }
    }

    /// Check if a status code should be considered a failure
    fn is_failure_status(&self, status: StatusCode) -> bool {
        self.failure_status_codes.contains(&status.as_u16())
    }
}

/// Layer for adding circuit breaker capability to services
#[derive(Clone, Debug)]
pub struct CircuitBreakerLayer {
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl CircuitBreakerLayer {
    /// Create a new circuit breaker layer with legacy consecutive failures behavior
    pub fn new(failure_threshold: u32, reset_timeout: Duration, success_threshold: u32) -> Self {
        Self::new_with_config(
            failure_threshold,
            reset_timeout,
            success_threshold,
            60,
            50,
            true,
            vec![500, 502, 503, 504],
        )
    }

    /// Create a new circuit breaker layer with full configuration
    pub fn new_with_config(
        failure_threshold: u32,
        reset_timeout: Duration,
        success_threshold: u32,
        window_seconds: u64,
        failure_percentage: u8,
        use_consecutive_failures: bool,
        failure_status_codes: Vec<u16>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState::new(
                failure_threshold,
                reset_timeout,
                success_threshold,
                window_seconds,
                failure_percentage,
                use_consecutive_failures,
                failure_status_codes,
            ))),
        }
    }
}

impl<S> Layer<S> for CircuitBreakerLayer {
    type Service = CircuitBreakerService<S>;

    fn layer(&self, service: S) -> Self::Service {
        CircuitBreakerService {
            inner: service,
            state: self.state.clone(),
        }
    }
}

/// Service implementing circuit breaker pattern
#[derive(Clone)]
pub struct CircuitBreakerService<S> {
    inner: S,
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for CircuitBreakerService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = axum::response::Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = CircuitBreakerFuture<S::Future, ResBody>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Check if the circuit is open
        {
            let mut state = self.state.lock().unwrap();
            state.check_transition_to_half_open();

            if state.state == CircuitState::Open {
                return Poll::Ready(Err("Circuit is open, requests are not allowed".into()));
            }
        }

        // If circuit is closed or half-open, check if the service is ready
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Check circuit state
        let current_state = {
            let mut state = self.state.lock().unwrap();
            state.check_transition_to_half_open();
            state.state
        };

        // If circuit is open, fail fast
        if current_state == CircuitState::Open {
            debug!(
                "Circuit is OPEN, failing fast for request to {}",
                req.uri().path()
            );
            return CircuitBreakerFuture {
                inner: futures::future::ready(Err(
                    "Circuit is open, requests are not allowed".into()
                ))
                .boxed(),
                state: self.state.clone(),
            };
        }

        // Call the inner service
        let path = req.uri().path().to_owned();
        debug!("Circuit is {}, allowing request to {}", current_state, path);
        let future = self.inner.call(req).map_err(Into::into);

        CircuitBreakerFuture {
            inner: future.boxed(),
            state: self.state.clone(),
        }
    }
}

/// Future that tracks the results of requests for circuit breaker state
#[pin_project]
pub struct CircuitBreakerFuture<F, ResBody> {
    #[pin]
    inner: futures::future::BoxFuture<
        'static,
        Result<axum::response::Response<ResBody>, Box<dyn std::error::Error + Send + Sync>>,
    >,
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl<F, ResBody> Future for CircuitBreakerFuture<F, ResBody>
where
    ResBody: Send + 'static,
{
    type Output =
        Result<axum::response::Response<ResBody>, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let inner = this.inner;
        let state = this.state;

        match inner.poll(cx) {
            Poll::Ready(Ok(response)) => {
                let status = response.status();
                let mut state_guard = state.lock().unwrap();

                // Check if this status code is considered a failure
                if state_guard.is_failure_status(status) {
                    state_guard.record_failure();
                    warn!(
                        "Circuit breaker recorded failure: status {} (circuit state: {})",
                        status, state_guard.state
                    );
                } else {
                    state_guard.record_success();
                    debug!(
                        "Circuit breaker recorded success (circuit state: {})",
                        state_guard.state
                    );
                }
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(err)) => {
                let mut state_guard = state.lock().unwrap();
                state_guard.record_failure();
                error!(
                    "Circuit breaker recorded error: {:?} (circuit state: {})",
                    err, state_guard.state
                );
                Poll::Ready(Err(err))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
