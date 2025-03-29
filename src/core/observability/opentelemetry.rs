use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram, ObservableGauge, Unit};
use opentelemetry::trace::{Span as OTelSpan, SpanBuilder, TraceContextExt, TraceId};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing::{error, info, warn};
use uuid::Uuid;

#[cfg(feature = "opentelemetry-jaeger")]
use opentelemetry::{global, runtime};
#[cfg(feature = "opentelemetry-jaeger")]
use opentelemetry_jaeger::{JaegerExporterBuilder, Propagator as JaegerPropagator};

#[cfg(feature = "otlp")]
use opentelemetry::sdk::export::metrics::aggregation;
#[cfg(feature = "otlp")]
use opentelemetry::sdk::metrics::reader;
#[cfg(feature = "otlp")]
use opentelemetry_otlp::{ExportConfig, WithExportConfig};

use crate::core::observability::config::ObservabilityConfig;
use crate::core::observability::error::ObservabilityError;
use crate::core::observability::operations::{
    MetricType, MetricValue, ObservabilityOperations, ProfilingSession, SpanContext, SpanStatus,
};
use crate::core::observability::provider::ObservabilityProvider;

/// OpenTelemetry client for tracing and metrics
pub struct OpenTelemetryClient {
    /// Service name
    service_name: String,
    /// Metrics registry
    metrics: HashMap<String, OpenTelemetryMetric>,
    /// Active spans
    active_spans: Arc<Mutex<HashMap<String, opentelemetry::trace::Span>>>,
    /// Profiling sessions
    profiling_sessions: Arc<Mutex<HashMap<String, ProfilingSession>>>,
    /// Initialization time
    init_time: Instant,
    /// Tracer provider
    tracer_provider: Option<opentelemetry::sdk::trace::TracerProvider>,
    /// Metrics provider
    metrics_provider: Option<opentelemetry::sdk::metrics::MeterProvider>,
    /// Correlation enabled
    correlation_enabled: bool,
}

/// OpenTelemetry metric type wrapper
enum OpenTelemetryMetric {
    /// Counter metric
    Counter(Counter<u64>),
    /// Gauge metric
    Gauge(ObservableGauge<f64>),
    /// Histogram metric
    Histogram(Histogram<f64>),
}

impl OpenTelemetryClient {
    /// Create a new OpenTelemetry client
    #[cfg(feature = "opentelemetry-jaeger")]
    pub fn new_jaeger(config: &ObservabilityConfig) -> Result<Self, ObservabilityError> {
        // Set up tracer
        let jaeger_endpoint = config
            .tracing_endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:14268/api/traces".to_string());

        let mut tracer_builder = JaegerExporterBuilder::default()
            .with_endpoint(jaeger_endpoint)
            .with_service_name(config.service_name.clone());

        // Add any custom configuration from provider config
        if let Some(agent_host) = config.provider_config.get("agent-host") {
            tracer_builder = tracer_builder.with_agent_endpoint(agent_host);
        }

        let tracer_provider = tracer_builder
            .build_batch_with_default_config()
            .map_err(|e| ObservabilityError::InitializationError(e.to_string()))?;

        // Install the tracer as global provider
        let _tracer = global::set_tracer_provider(tracer_provider.clone());

        // Create a client instance
        Ok(Self {
            service_name: config.service_name.clone(),
            metrics: HashMap::new(),
            active_spans: Arc::new(Mutex::new(HashMap::new())),
            profiling_sessions: Arc::new(Mutex::new(HashMap::new())),
            init_time: Instant::now(),
            tracer_provider: Some(tracer_provider),
            metrics_provider: None,
            correlation_enabled: config.correlation_enabled,
        })
    }

    /// Create a new OpenTelemetry client with OTLP exporter
    #[cfg(feature = "otlp")]
    pub fn new_otlp(config: &ObservabilityConfig) -> Result<Self, ObservabilityError> {
        // Set up tracer
        let otlp_endpoint = config
            .tracing_endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:4317".to_string());

        // Set up metrics
        let export_config = ExportConfig {
            endpoint: otlp_endpoint.clone(),
            ..ExportConfig::default()
        };

        // Create a meter provider
        let meter_provider = opentelemetry_otlp::new_pipeline()
            .metrics(opentelemetry::runtime::Tokio)
            .with_export_config(export_config.clone())
            .with_resource(vec![KeyValue::new(
                SERVICE_NAME,
                config.service_name.clone(),
            )])
            .build()
            .map_err(|e| ObservabilityError::InitializationError(e.to_string()))?;

        // Set up tracer
        let tracer_provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_endpoint(otlp_endpoint),
            )
            .with_resource(vec![KeyValue::new(
                SERVICE_NAME,
                config.service_name.clone(),
            )])
            .install_batch(opentelemetry::runtime::Tokio)
            .map_err(|e| ObservabilityError::InitializationError(e.to_string()))?;

        // Create a client instance
        Ok(Self {
            service_name: config.service_name.clone(),
            metrics: HashMap::new(),
            active_spans: Arc::new(Mutex::new(HashMap::new())),
            profiling_sessions: Arc::new(Mutex::new(HashMap::new())),
            init_time: Instant::now(),
            tracer_provider: Some(tracer_provider),
            metrics_provider: Some(meter_provider),
            correlation_enabled: config.correlation_enabled,
        })
    }

    /// Create prefixed metric name
    fn create_metric_name(&self, name: &str) -> String {
        format!("{}_{}", self.service_name, name)
    }

    /// Convert label pairs to OpenTelemetry attributes
    fn labels_to_attributes(&self, labels: &[(&str, String)]) -> Vec<KeyValue> {
        labels
            .iter()
            .map(|(k, v)| KeyValue::new(*k, v.clone()))
            .collect()
    }
}

impl ObservabilityOperations for OpenTelemetryClient {
    fn record_counter(
        &self,
        name: &str,
        value: u64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError> {
        let metric_name = self.create_metric_name(name);
        let attributes = self.labels_to_attributes(labels);

        // Use OpenTelemetry metrics if available, otherwise log
        if let Some(ref meter_provider) = self.metrics_provider {
            let meter = meter_provider.meter("navius");

            // Get or create counter
            let counter = self
                .metrics
                .get(&metric_name)
                .map(|m| match m {
                    OpenTelemetryMetric::Counter(c) => Ok(c.clone()),
                    _ => Err(ObservabilityError::MetricError(format!(
                        "Metric {} exists but is not a counter",
                        metric_name
                    ))),
                })
                .unwrap_or_else(|| {
                    // Create new counter
                    let counter = meter
                        .u64_counter(metric_name.clone())
                        .with_description(format!("Counter for {}", name))
                        .init();

                    Ok(counter)
                })?;

            counter.add(value, &attributes);
        } else {
            // Fallback to logging if no metrics provider
            info!(
                counter = metric_name,
                value = value,
                labels = ?labels,
                "Recording counter metric"
            );
        }

        Ok(())
    }

    fn record_gauge(
        &self,
        name: &str,
        value: f64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError> {
        let metric_name = self.create_metric_name(name);
        let attributes = self.labels_to_attributes(labels);

        // Use OpenTelemetry metrics if available, otherwise log
        if let Some(ref meter_provider) = self.metrics_provider {
            // For gauge, we need to use observable gauge with a callback
            // This is a simplification - proper implementation would track values by attributes
            let meter = meter_provider.meter("navius");

            info!(
                gauge = metric_name,
                value = value,
                labels = ?labels,
                "Recording gauge metric"
            );

            // In a real implementation, we would register gauge observers
            // and track values by attribute sets
        } else {
            // Fallback to logging if no metrics provider
            info!(
                gauge = metric_name,
                value = value,
                labels = ?labels,
                "Recording gauge metric"
            );
        }

        Ok(())
    }

    fn record_histogram(
        &self,
        name: &str,
        value: f64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError> {
        let metric_name = self.create_metric_name(name);
        let attributes = self.labels_to_attributes(labels);

        // Use OpenTelemetry metrics if available, otherwise log
        if let Some(ref meter_provider) = self.metrics_provider {
            let meter = meter_provider.meter("navius");

            // Get or create histogram
            let histogram = self
                .metrics
                .get(&metric_name)
                .map(|m| match m {
                    OpenTelemetryMetric::Histogram(h) => Ok(h.clone()),
                    _ => Err(ObservabilityError::MetricError(format!(
                        "Metric {} exists but is not a histogram",
                        metric_name
                    ))),
                })
                .unwrap_or_else(|| {
                    // Create new histogram
                    let histogram = meter
                        .f64_histogram(metric_name.clone())
                        .with_description(format!("Histogram for {}", name))
                        .with_unit(Unit::new("seconds"))
                        .init();

                    Ok(histogram)
                })?;

            histogram.record(value, &attributes);
        } else {
            // Fallback to logging if no metrics provider
            info!(
                histogram = metric_name,
                value = value,
                labels = ?labels,
                "Recording histogram metric"
            );
        }

        Ok(())
    }

    fn get_metric(
        &self,
        name: &str,
        metric_type: MetricType,
        _labels: &[(&str, String)],
    ) -> Result<Option<MetricValue>, ObservabilityError> {
        // OpenTelemetry doesn't provide a way to retrieve current metric values
        warn!("OpenTelemetry doesn't support retrieving metric values directly");

        // Return placeholder values
        match metric_type {
            MetricType::Counter => Ok(Some(MetricValue::Counter(0))),
            MetricType::Gauge => Ok(Some(MetricValue::Gauge(0.0))),
            MetricType::Histogram => Ok(Some(MetricValue::Histogram(0.0))),
        }
    }

    fn start_span(&self, name: &str) -> SpanContext {
        let span_id = Uuid::new_v4().to_string();
        let trace_id = Uuid::new_v4().to_string();

        // Create OpenTelemetry span if tracer is available
        if let Some(ref tracer_provider) = self.tracer_provider {
            let tracer = tracer_provider.tracer("navius");
            let span = tracer.start(name);

            // Store the OpenTelemetry span
            if self.correlation_enabled {
                if let Ok(mut spans) = self.active_spans.lock() {
                    spans.insert(span_id.clone(), span);
                }
            }
        }

        // Return our span context
        SpanContext {
            span_id,
            trace_id,
            name: name.to_string(),
            start_time: Instant::now(),
            attributes: Vec::new(),
        }
    }

    fn end_span(&self, context: SpanContext) {
        // Calculate duration
        let duration = context.start_time.elapsed().as_secs_f64();

        // End the OpenTelemetry span if it exists
        if self.correlation_enabled {
            if let Ok(mut spans) = self.active_spans.lock() {
                if let Some(span) = spans.remove(&context.span_id) {
                    span.end();
                }
            }
        }

        // Record duration as a histogram
        let labels = vec![
            ("name", context.name.clone()),
            ("span_id", context.span_id),
            ("trace_id", context.trace_id),
        ];

        if let Err(e) = self.record_histogram("span_duration_seconds", duration, &labels) {
            error!("Failed to record span duration: {}", e);
        }
    }

    fn set_span_attribute(&self, context: &SpanContext, key: &str, value: &str) {
        // Set attribute on OpenTelemetry span if available
        if self.correlation_enabled {
            if let Ok(spans) = self.active_spans.lock() {
                if let Some(span) = spans.get(&context.span_id) {
                    span.set_attribute(KeyValue::new(key, value.to_string()));
                }
            }
        }
    }

    fn set_span_status(
        &self,
        context: &SpanContext,
        status: SpanStatus,
        description: Option<&str>,
    ) {
        // Set status on OpenTelemetry span if available
        if self.correlation_enabled {
            if let Ok(spans) = self.active_spans.lock() {
                if let Some(span) = spans.get(&context.span_id) {
                    let otel_status = match status {
                        SpanStatus::Ok => opentelemetry::trace::Status::Ok,
                        SpanStatus::Error => {
                            let desc = description.unwrap_or("Error");
                            opentelemetry::trace::Status::error(desc.to_string())
                        }
                        SpanStatus::Canceled => {
                            let desc = description.unwrap_or("Canceled");
                            opentelemetry::trace::Status::error(desc.to_string())
                        }
                    };
                    span.set_status(otel_status);
                }
            }
        }

        // Record as a metric as well
        let status_name = match status {
            SpanStatus::Ok => "ok",
            SpanStatus::Error => "error",
            SpanStatus::Canceled => "canceled",
        };

        let mut labels = vec![
            ("name", context.name.clone()),
            ("span_id", context.span_id.clone()),
            ("trace_id", context.trace_id.clone()),
            ("status", status_name.to_string()),
        ];

        if let Some(desc) = description {
            labels.push(("description", desc.to_string()));
        }

        if let Err(e) = self.record_counter("span_status_total", 1, &labels) {
            error!("Failed to record span status: {}", e);
        }
    }

    fn start_profiling(&self, name: &str) -> Result<ProfilingSession, ObservabilityError> {
        let session = ProfilingSession::new(name);
        let id = session.id.clone();

        // Store the session
        if let Ok(mut sessions) = self.profiling_sessions.lock() {
            sessions.insert(id.clone(), session.clone());
        }

        Ok(session)
    }

    fn health_check(&self) -> Result<bool, ObservabilityError> {
        // Simple health check - checks if the client is initialized
        Ok(self.init_time.elapsed().as_secs() > 0)
    }
}

/// OpenTelemetry provider - base implementation
pub struct OpenTelemetryProvider {
    /// Provider name
    name: String,
}

impl OpenTelemetryProvider {
    /// Create a new OpenTelemetry provider
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// Jaeger provider
pub struct JaegerProvider;

impl JaegerProvider {
    /// Create a new Jaeger provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ObservabilityProvider for JaegerProvider {
    async fn create_client(
        &self,
        config: ObservabilityConfig,
    ) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError> {
        #[cfg(feature = "opentelemetry-jaeger")]
        {
            let client = OpenTelemetryClient::new_jaeger(&config)?;
            return Ok(Box::new(client));
        }

        #[cfg(not(feature = "opentelemetry-jaeger"))]
        {
            return Err(ObservabilityError::ProviderNotSupported(
                "Jaeger support is not enabled in this build. Recompile with the 'opentelemetry-jaeger' feature".to_string()
            ));
        }
    }

    fn supports(&self, config: &ObservabilityConfig) -> bool {
        config.provider == "jaeger"
    }

    fn name(&self) -> &str {
        "jaeger"
    }
}

/// OpenTelemetry Protocol (OTLP) provider
pub struct OtlpProvider;

impl OtlpProvider {
    /// Create a new OTLP provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ObservabilityProvider for OtlpProvider {
    async fn create_client(
        &self,
        config: ObservabilityConfig,
    ) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError> {
        #[cfg(feature = "otlp")]
        {
            let client = OpenTelemetryClient::new_otlp(&config)?;
            return Ok(Box::new(client));
        }

        #[cfg(not(feature = "otlp"))]
        {
            return Err(ObservabilityError::ProviderNotSupported(
                "OTLP support is not enabled in this build. Recompile with the 'otlp' feature"
                    .to_string(),
            ));
        }
    }

    fn supports(&self, config: &ObservabilityConfig) -> bool {
        config.provider == "otlp"
    }

    fn name(&self) -> &str {
        "otlp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "opentelemetry-jaeger")]
    async fn test_jaeger_provider() {
        let provider = JaegerProvider::new();
        let config = ObservabilityConfig::new("jaeger", "test-service");

        assert!(provider.supports(&config));
        assert_eq!(provider.name(), "jaeger");

        let client_result = provider.create_client(config).await;
        assert!(client_result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "otlp")]
    async fn test_otlp_provider() {
        let provider = OtlpProvider::new();
        let config = ObservabilityConfig::new("otlp", "test-service");

        assert!(provider.supports(&config));
        assert_eq!(provider.name(), "otlp");

        let client_result = provider.create_client(config).await;
        assert!(client_result.is_ok());
    }
}
