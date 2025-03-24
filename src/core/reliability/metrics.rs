use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use axum::response::Response;
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt};
use metrics::{counter, gauge, histogram};
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{debug, info};

/// Reliability metrics for monitoring
#[derive(Debug, Clone)]
pub struct ReliabilityMetrics {
    /// Total number of requests processed
    total_requests: Arc<AtomicU64>,
    /// Number of successful requests (2xx responses)
    successful_requests: Arc<AtomicU64>,
    /// Number of client errors (4xx responses)
    client_errors: Arc<AtomicU64>,
    /// Number of server errors (5xx responses)
    server_errors: Arc<AtomicU64>,
    /// Number of timed out requests
    timeouts: Arc<AtomicU64>,
    /// Number of rate limited requests
    rate_limited: Arc<AtomicU64>,
    /// Number of circuit broken requests
    circuit_broken: Arc<AtomicU64>,
    /// Number of retry attempts
    retry_attempts: Arc<AtomicU64>,
}

impl ReliabilityMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            client_errors: Arc::new(AtomicU64::new(0)),
            server_errors: Arc::new(AtomicU64::new(0)),
            timeouts: Arc::new(AtomicU64::new(0)),
            rate_limited: Arc::new(AtomicU64::new(0)),
            circuit_broken: Arc::new(AtomicU64::new(0)),
            retry_attempts: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a new request
    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a response based on status code
    pub fn record_response(&self, status: StatusCode) {
        if status.is_success() {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else if status.is_client_error() {
            if status == StatusCode::TOO_MANY_REQUESTS {
                self.rate_limited.fetch_add(1, Ordering::Relaxed);
            }
            self.client_errors.fetch_add(1, Ordering::Relaxed);
        } else if status.is_server_error() {
            if status == StatusCode::SERVICE_UNAVAILABLE {
                self.circuit_broken.fetch_add(1, Ordering::Relaxed);
            }
            self.server_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record a timeout
    pub fn record_timeout(&self) {
        self.timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a retry attempt
    pub fn record_retry(&self) {
        self.retry_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// Update Prometheus metrics
    pub fn update_prometheus_metrics(&self) {
        // Update counters with actual values
        counter!("reliability.requests.total", "type" => "total")
            .increment(self.total_requests.load(Ordering::Relaxed));
        counter!("reliability.requests.successful", "type" => "successful")
            .increment(self.successful_requests.load(Ordering::Relaxed));
        counter!("reliability.requests.client_errors", "type" => "client_errors")
            .increment(self.client_errors.load(Ordering::Relaxed));
        counter!("reliability.requests.server_errors", "type" => "server_errors")
            .increment(self.server_errors.load(Ordering::Relaxed));
        counter!("reliability.requests.timeouts", "type" => "timeouts")
            .increment(self.timeouts.load(Ordering::Relaxed));
        counter!("reliability.requests.rate_limited", "type" => "rate_limited")
            .increment(self.rate_limited.load(Ordering::Relaxed));
        counter!("reliability.requests.circuit_broken", "type" => "circuit_broken")
            .increment(self.circuit_broken.load(Ordering::Relaxed));
        counter!("reliability.requests.retry_attempts", "type" => "retry_attempts")
            .increment(self.retry_attempts.load(Ordering::Relaxed));

        // Calculate error rate
        let total_requests = self.total_requests.load(Ordering::Relaxed) as f64;
        if total_requests > 0.0 {
            let error_requests = (self.client_errors.load(Ordering::Relaxed)
                + self.server_errors.load(Ordering::Relaxed)
                + self.timeouts.load(Ordering::Relaxed)
                + self.rate_limited.load(Ordering::Relaxed)
                + self.circuit_broken.load(Ordering::Relaxed))
                as f64;
            let error_rate = error_requests / total_requests;
            metrics::gauge!("reliability.error_rate").set(error_rate);
        }
    }
}

impl Default for ReliabilityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ReliabilityMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.total_requests.load(Ordering::Relaxed);
        let success = self.successful_requests.load(Ordering::Relaxed);
        let client_err = self.client_errors.load(Ordering::Relaxed);
        let server_err = self.server_errors.load(Ordering::Relaxed);
        let rate_limited = self.rate_limited.load(Ordering::Relaxed);
        let circuit_broken = self.circuit_broken.load(Ordering::Relaxed);
        let timeouts = self.timeouts.load(Ordering::Relaxed);
        let retries = self.retry_attempts.load(Ordering::Relaxed);

        let success_rate = if total > 0 {
            (success as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        write!(
            f,
            "Requests: {}, Success: {} ({:.1}%), Client Errors: {}, Server Errors: {}, Rate Limited: {}, Circuit Broken: {}, Timeouts: {}, Retries: {}",
            total,
            success,
            success_rate,
            client_err,
            server_err,
            rate_limited,
            circuit_broken,
            timeouts,
            retries
        )
    }
}

/// Layer for adding reliability metrics
#[derive(Clone)]
pub struct ReliabilityMetricsLayer {
    metrics: Arc<ReliabilityMetrics>,
    update_interval: Duration,
}

impl ReliabilityMetricsLayer {
    /// Create a new metrics layer with default settings
    pub fn new() -> Self {
        Self::with_update_interval(Duration::from_secs(15))
    }

    /// Create a new metrics layer with a custom update interval
    pub fn with_update_interval(update_interval: Duration) -> Self {
        let metrics = Arc::new(ReliabilityMetrics::new());

        // Schedule periodic updates to Prometheus metrics
        let metrics_clone = metrics.clone();
        let interval = update_interval;

        info!(
            "Initializing reliability metrics with update interval of {:?}",
            interval
        );

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                metrics_clone.update_prometheus_metrics();
                debug!("Updated reliability metrics: {}", metrics_clone);
            }
        });

        Self {
            metrics,
            update_interval,
        }
    }

    /// Get a reference to the metrics
    pub fn metrics(&self) -> Arc<ReliabilityMetrics> {
        self.metrics.clone()
    }

    /// Get the configured update interval
    pub fn update_interval(&self) -> Duration {
        self.update_interval
    }
}

impl<S> Layer<S> for ReliabilityMetricsLayer {
    type Service = ReliabilityMetricsService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ReliabilityMetricsService {
            inner: service,
            metrics: self.metrics.clone(),
        }
    }
}

impl Default for ReliabilityMetricsLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Service that collects reliability metrics
#[derive(Clone)]
pub struct ReliabilityMetricsService<S> {
    inner: S,
    metrics: Arc<ReliabilityMetrics>,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for ReliabilityMetricsService<S>
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
        self.inner.poll_ready(cx).map_err(|e| Box::new(e) as _)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Record the start time
        let start = Instant::now();
        let path = req.uri().path().to_string();

        // Increment request counter
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        counter!("reliability.requests.total").increment(1);

        // Get a clone of the service to handle the request
        let clone_service = self.inner.clone();
        let mut service = std::mem::replace(&mut self.inner, clone_service);

        // Clone metrics for use in the future
        let metrics = self.metrics.clone();

        // Call the inner service
        let future = service.call(req);

        async move {
            let result = future.await;

            // Record request duration regardless of success/failure
            let duration = start.elapsed();
            let duration_ms = duration.as_millis() as f64;
            metrics::histogram!("reliability.request.duration").record(duration_ms);

            // Handle the result
            match result {
                Ok(response) => {
                    // Record metrics based on response status
                    let status = response.status();
                    let status_code = status.as_u16();

                    if status.is_success() {
                        metrics.successful_requests.fetch_add(1, Ordering::Relaxed);
                        counter!("reliability.requests.successful").increment(1);
                    } else if status.is_client_error() {
                        metrics.client_errors.fetch_add(1, Ordering::Relaxed);
                        counter!("reliability.requests.client_errors").increment(1);

                        // Special handling for rate limiting
                        if status == StatusCode::TOO_MANY_REQUESTS {
                            metrics.rate_limited.fetch_add(1, Ordering::Relaxed);
                            counter!("reliability.requests.rate_limited").increment(1);
                        }
                    } else if status.is_server_error() {
                        metrics.server_errors.fetch_add(1, Ordering::Relaxed);
                        counter!("reliability.requests.server_errors").increment(1);

                        // Special handling for circuit breaking
                        if status == StatusCode::SERVICE_UNAVAILABLE {
                            metrics.circuit_broken.fetch_add(1, Ordering::Relaxed);
                            counter!("reliability.requests.circuit_broken").increment(1);
                        }
                    }

                    // Calculate and update error rate
                    let total_requests = metrics.total_requests.load(Ordering::Relaxed) as f64;
                    let error_requests = (metrics.client_errors.load(Ordering::Relaxed)
                        + metrics.server_errors.load(Ordering::Relaxed))
                        as f64;

                    if total_requests > 0.0 {
                        let error_rate = error_requests / total_requests;
                        metrics::gauge!("reliability.error_rate").set(error_rate);
                    }

                    debug!(
                        "Request completed: path={}, status={}, duration={:?}",
                        path, status_code, duration
                    );

                    Ok(response)
                }
                Err(e) => {
                    // Record server error for any service errors
                    metrics.server_errors.fetch_add(1, Ordering::Relaxed);
                    counter!("reliability.requests.server_errors").increment(1);

                    // Log the error with context
                    tracing::error!("Service error for path {}: {}", path, e);

                    Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }
            }
        }
        .boxed()
    }
}
