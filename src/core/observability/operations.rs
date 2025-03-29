use std::fmt;
use std::time::Duration;

use crate::core::observability::error::ObservabilityError;

/// Metric value types
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Counter value (always increases)
    Counter(u64),
    /// Gauge value (can increase and decrease)
    Gauge(f64),
    /// Histogram value (statistical distribution)
    Histogram(f64),
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::Counter(val) => write!(f, "{}", val),
            MetricValue::Gauge(val) => write!(f, "{}", val),
            MetricValue::Histogram(val) => write!(f, "{}", val),
        }
    }
}

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Counter metric
    Counter,
    /// Gauge metric
    Gauge,
    /// Histogram metric
    Histogram,
}

/// Span context for tracing
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Span ID
    pub span_id: String,
    /// Trace ID
    pub trace_id: String,
    /// Span name
    pub name: String,
    /// Span start time
    pub start_time: std::time::Instant,
    /// Span attributes
    pub attributes: Vec<(String, String)>,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus {
    /// Span completed successfully
    Ok,
    /// Span completed with an error
    Error,
    /// Span was canceled
    Canceled,
}

/// Profiling session
#[derive(Debug)]
pub struct ProfilingSession {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: String,
    /// Session start time
    pub start_time: std::time::Instant,
    /// Whether the session is running
    pub running: bool,
}

impl ProfilingSession {
    /// Create a new profiling session
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            start_time: std::time::Instant::now(),
            running: true,
        }
    }

    /// Stop the profiling session
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get the duration of the session
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Trait for observability operations
pub trait ObservabilityOperations: Send + Sync + 'static {
    /// Record a counter metric
    fn record_counter(
        &self,
        name: &str,
        value: u64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError>;

    /// Record a gauge metric
    fn record_gauge(
        &self,
        name: &str,
        value: f64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError>;

    /// Record a histogram metric
    fn record_histogram(
        &self,
        name: &str,
        value: f64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError>;

    /// Get a metric value
    fn get_metric(
        &self,
        name: &str,
        metric_type: MetricType,
        labels: &[(&str, String)],
    ) -> Result<Option<MetricValue>, ObservabilityError>;

    /// Start a span for distributed tracing
    fn start_span(&self, name: &str) -> SpanContext;

    /// End a span
    fn end_span(&self, context: SpanContext);

    /// Set an attribute on a span
    fn set_span_attribute(&self, context: &SpanContext, key: &str, value: &str);

    /// Set the status of a span
    fn set_span_status(&self, context: &SpanContext, status: SpanStatus, description: Option<&str>);

    /// Start a profiling session
    fn start_profiling(&self, name: &str) -> Result<ProfilingSession, ObservabilityError>;

    /// Health check for the observability system
    fn health_check(&self) -> Result<bool, ObservabilityError>;
}
