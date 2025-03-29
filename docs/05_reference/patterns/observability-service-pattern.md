# Observability Service Pattern

This pattern provides a unified approach to observability in Navius applications, including metrics, distributed tracing, and profiling. The pattern follows the provider model, allowing different implementations to be used based on configuration.

## Core Components

### ObservabilityOperations Interface

The `ObservabilityOperations` trait defines the operations that can be performed:

```rust
pub trait ObservabilityOperations: Send + Sync + 'static {
    // Metrics operations
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, String)]) -> Result<(), ObservabilityError>;
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, String)]) -> Result<(), ObservabilityError>;
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, String)]) -> Result<(), ObservabilityError>;
    fn get_metric(&self, name: &str, metric_type: MetricType, labels: &[(&str, String)]) -> Result<Option<MetricValue>, ObservabilityError>;
    
    // Tracing operations
    fn start_span(&self, name: &str) -> SpanContext;
    fn end_span(&self, context: SpanContext);
    fn set_span_attribute(&self, context: &SpanContext, key: &str, value: &str);
    fn set_span_status(&self, context: &SpanContext, status: SpanStatus, description: Option<&str>);
    
    // Profiling operations
    fn start_profiling(&self, name: &str) -> Result<ProfilingSession, ObservabilityError>;
    
    // Health check
    fn health_check(&self) -> Result<bool, ObservabilityError>;
}
```

### ObservabilityProvider Interface

The `ObservabilityProvider` trait defines how observability implementations are created:

```rust
#[async_trait]
pub trait ObservabilityProvider: Send + Sync + 'static {
    async fn create_client(&self, config: ObservabilityConfig) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError>;
    fn supports(&self, config: &ObservabilityConfig) -> bool;
    fn name(&self) -> &str;
}
```

### ObservabilityProviderRegistry

The registry manages all available providers:

```rust
pub struct ObservabilityProviderRegistry {
    providers: HashMap<String, Arc<dyn ObservabilityProvider>>,
    default_provider: Option<String>,
}
```

### ObservabilityService

The service is the main entry point for application code:

```rust
pub struct ObservabilityService {
    client: Arc<dyn ObservabilityOperations>,
    registry: Arc<ObservabilityProviderRegistry>,
    config: ObservabilityConfig,
}
```

## Key Concepts

1. **Providers**: Implementations of observability tools like Prometheus, Dynatrace, or OpenTelemetry
2. **Operations**: The actual metrics recording, tracing, and profiling capabilities
3. **Registry**: A collection of available providers that can be selected at runtime
4. **Service**: The facade that application code interacts with

## Standard Implementations

1. **Prometheus Provider**: Uses the metrics-exporter-prometheus crate for metrics collection
2. **Dynatrace Provider**: (Planned) Integration with Dynatrace monitoring
3. **OpenTelemetry Provider**: (Planned) Implementation using the OpenTelemetry standard

## How to Use

### Basic Usage

```rust
use navius::core::observability::{init_observability, MetricType};

async fn main() {
    // Initialize with default provider (Prometheus)
    let observability = init_observability("my-service").await.unwrap();
    
    // Record metrics
    observability.record_counter("requests_total", 1).unwrap();
    observability.record_gauge("memory_usage", 1024.0).unwrap();
    
    // Record with labels
    let labels = vec![("endpoint", "/api/users".to_string())];
    observability.record_counter_with_labels("endpoint_requests", 1, &labels).unwrap();
    
    // Create a span for tracing
    let span = observability.start_span("process_request");
    
    // Do some work
    // ...
    
    // Set span attributes
    observability.set_span_attribute(&span, "user_id", "12345");
    
    // End the span
    observability.end_span(span);
}
```

### Custom Provider Configuration

```rust
use navius::core::observability::{
    ObservabilityConfig, ObservabilityProviderRegistry, ObservabilityService,
    PrometheusProvider
};
use std::time::Duration;

async fn setup_custom_observability() -> ObservabilityService {
    // Create registry
    let mut registry = ObservabilityProviderRegistry::new();
    
    // Register providers
    registry.register(PrometheusProvider::new());
    // registry.register(DynatraceProvider::new());
    
    // Set default provider
    registry.set_default_provider("prometheus").unwrap();
    
    // Create custom configuration
    let config = ObservabilityConfig::new("prometheus", "my-service")
        .with_export_interval(Duration::from_secs(30))
        .with_environment("production")
        .with_tracing_enabled(true)
        .with_profiling_enabled(true)
        .with_trace_sample_rate(0.1);
    
    // Create service
    ObservabilityService::new(registry, config).await.unwrap()
}
```

## Best Practices

1. **Consistent Naming**: Use a consistent naming scheme for metrics (e.g., snake_case with service prefix)
2. **Meaningful Labels**: Add labels to provide context but avoid high cardinality
3. **Record Appropriate Metric Types**:
   - Counters: For values that only increase (e.g., requests_total)
   - Gauges: For values that can go up and down (e.g., memory_usage)
   - Histograms: For measuring distributions (e.g., request_duration_seconds)
4. **Sample Traces Appropriately**: Don't trace every request in production
5. **Health Checks**: Include observability system in health checks
6. **Error Handling**: Always handle errors from metrics recording

## Error Handling

All observability operations return a `Result<T, ObservabilityError>` that should be handled appropriately:

```rust
match observability.record_counter("requests_total", 1) {
    Ok(_) => {}, // Metrics recorded successfully
    Err(e) => {
        // Log the error but don't crash the application
        eprintln!("Failed to record metric: {}", e);
    }
}
```

## Integration with Health Checks

The observability system can be included in health checks:

```rust
async fn check_health(observability: &ObservabilityService) -> bool {
    match observability.health_check() {
        Ok(true) => true,
        _ => false,
    }
}
```

## Extending with Custom Providers

To create a custom provider:

1. Implement the `ObservabilityOperations` trait for your client
2. Implement the `ObservabilityProvider` trait to create your client
3. Register your provider with the registry

Example:

```rust
#[async_trait]
impl ObservabilityProvider for MyCustomProvider {
    async fn create_client(&self, config: ObservabilityConfig) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError> {
        let client = MyCustomClient::new(&config)?;
        Ok(Box::new(client))
    }
    
    fn supports(&self, config: &ObservabilityConfig) -> bool {
        config.provider == "my-custom"
    }
    
    fn name(&self) -> &str {
        "my-custom"
    }
} 