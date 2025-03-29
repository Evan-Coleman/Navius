# Observability Service Example

This example shows how to use the observability service in your Navius application to record metrics, create traces, and measure performance.

## Basic Setup

First, import the necessary types and initialize the observability service:

```rust
use navius::core::observability::{
    init_observability, MetricType, ObservabilityConfig, ObservabilityProviderRegistry,
    ObservabilityService, PrometheusProvider, SpanStatus
};

async fn setup_observability() -> ObservabilityService {
    // Initialize with default setup (Prometheus provider)
    init_observability("my-service").await.unwrap()
}
```

## Recording Metrics

You can record different types of metrics:

```rust
async fn record_metrics(observability: &ObservabilityService) {
    // Record a simple counter
    observability.record_counter("api_requests_total", 1).unwrap();
    
    // Record a gauge with the current value
    observability.record_gauge("memory_usage_bytes", 1_024_000.0).unwrap();
    
    // Record a histogram for measuring distributions
    observability.record_histogram("request_duration_seconds", 0.42).unwrap();
    
    // Record metrics with labels
    let labels = vec![
        ("endpoint", "/api/users".to_string()),
        ("method", "GET".to_string()),
    ];
    
    observability.record_counter_with_labels("endpoint_requests", 1, &labels).unwrap();
}
```

## Creating Spans for Distributed Tracing

Spans allow you to track the execution of operations across your application:

```rust
async fn process_request(observability: &ObservabilityService, user_id: &str) {
    // Start a span for this operation
    let span = observability.start_span("process_request");
    
    // Add attributes to the span
    observability.set_span_attribute(&span, "user_id", user_id);
    
    // Perform some work
    let result = do_some_work(user_id).await;
    
    // Set the span status based on the result
    if result.is_ok() {
        observability.set_span_status(&span, SpanStatus::Ok, None);
    } else {
        observability.set_span_status(
            &span, 
            SpanStatus::Error, 
            Some("Failed to process request")
        );
    }
    
    // End the span - this records its duration
    observability.end_span(span);
}

async fn do_some_work(user_id: &str) -> Result<(), String> {
    // Simulate some work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Simulate a success
    Ok(())
}
```

## Nested Spans

You can create nested spans to track sub-operations:

```rust
async fn process_complex_request(observability: &ObservabilityService, user_id: &str) {
    // Start a parent span
    let parent_span = observability.start_span("process_complex_request");
    observability.set_span_attribute(&parent_span, "user_id", user_id);
    
    // Perform the first sub-operation
    let validate_span = observability.start_span("validate_user");
    observability.set_span_attribute(&validate_span, "user_id", user_id);
    
    // Simulate validation
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // End the validation span
    observability.set_span_status(&validate_span, SpanStatus::Ok, None);
    observability.end_span(validate_span);
    
    // Perform the second sub-operation
    let fetch_span = observability.start_span("fetch_user_data");
    observability.set_span_attribute(&fetch_span, "user_id", user_id);
    
    // Simulate data fetch
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    
    // End the fetch span
    observability.set_span_status(&fetch_span, SpanStatus::Ok, None);
    observability.end_span(fetch_span);
    
    // End the parent span
    observability.set_span_status(&parent_span, SpanStatus::Ok, None);
    observability.end_span(parent_span);
}
```

## Profiling

You can use profiling to measure the performance of specific operations:

```rust
async fn profile_operation(observability: &ObservabilityService) -> Result<(), String> {
    // Start a profiling session
    let mut session = match observability.start_profiling("expensive_operation") {
        Ok(session) => session,
        Err(e) => return Err(format!("Failed to start profiling: {}", e)),
    };
    
    // Perform some expensive operation
    expensive_operation().await;
    
    // Get the duration and stop the session
    let duration = session.duration();
    println!("Operation took: {:?}", duration);
    
    Ok(())
}

async fn expensive_operation() {
    // Simulate an expensive operation
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

## Custom Configuration

You can configure the observability service with custom settings:

```rust
use std::time::Duration;

async fn setup_custom_observability() -> ObservabilityService {
    // Create a registry
    let mut registry = ObservabilityProviderRegistry::new();
    
    // Register the Prometheus provider
    registry.register(PrometheusProvider::new());
    
    // Set it as the default provider
    registry.set_default_provider("prometheus").unwrap();
    
    // Create a custom configuration
    let config = ObservabilityConfig::new("prometheus", "my-custom-service")
        .with_export_enabled(true)
        .with_export_interval(Duration::from_secs(15))
        .with_environment("staging")
        .with_tracing_enabled(true)
        .with_profiling_enabled(true)
        .with_trace_sample_rate(0.25);
    
    // Create the service with the registry and configuration
    ObservabilityService::new(registry, config).await.unwrap()
}
```

## Integration with Request Handlers

Here's how to integrate observability with Axum request handlers:

```rust
use axum::{
    extract::State,
    routing::get,
    Router,
};
use std::sync::Arc;

struct AppState {
    observability: ObservabilityService,
}

async fn main() {
    // Set up observability
    let observability = init_observability("web-service").await.unwrap();
    
    // Create shared state
    let state = Arc::new(AppState { observability });
    
    // Build your router
    let app = Router::new()
        .route("/", get(handle_request))
        .with_state(state);
    
    // Run the server
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_request(State(state): State<Arc<AppState>>) -> String {
    // Start a span for this request
    let span = state.observability.start_span("handle_request");
    
    // Record a metric
    state.observability.record_counter("requests_total", 1).unwrap();
    
    // Process the request
    let result = "Hello, World!";
    
    // Record the response size
    let labels = vec![("endpoint", "/".to_string())];
    state.observability.record_histogram_with_labels(
        "response_size_bytes", 
        result.len() as f64, 
        &labels
    ).unwrap();
    
    // End the span
    state.observability.set_span_status(&span, SpanStatus::Ok, None);
    state.observability.end_span(span);
    
    result.to_string()
}
```

## Error Handling

Always handle errors from observability operations to prevent them from affecting your application:

```rust
fn record_metric_safely(observability: &ObservabilityService, name: &str, value: u64) {
    match observability.record_counter(name, value) {
        Ok(_) => {}
        Err(e) => {
            // Log the error but don't let it affect the application
            eprintln!("Failed to record metric {}: {}", name, e);
        }
    }
}
```

## Health Checks

Include the observability system in your application's health checks:

```rust
async fn health_check(observability: &ObservabilityService) -> bool {
    match observability.health_check() {
        Ok(true) => {
            println!("Observability system is healthy");
            true
        }
        Ok(false) => {
            println!("Observability system is unhealthy");
            false
        }
        Err(e) => {
            println!("Failed to check observability health: {}", e);
            false
        }
    }
}
```

## Complete Example

Here's a complete example that puts everything together:

```rust
use navius::core::observability::{
    init_observability, MetricType, SpanStatus
};
use std::sync::Arc;
use axum::{
    extract::State,
    routing::get,
    Router,
};
use tokio::time::Duration;

struct AppState {
    observability: Arc<observability::ObservabilityService>,
}

#[tokio::main]
async fn main() {
    // Initialize observability
    let observability = Arc::new(init_observability("example-service").await.unwrap());
    
    // Create shared state
    let state = AppState {
        observability: observability.clone(),
    };
    
    // Build router
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/users/:id", get(handle_get_user))
        .route("/health", get(handle_health))
        .with_state(state);
    
    // Start recording application metrics
    tokio::spawn(record_app_metrics(observability.clone()));
    
    // Run the server
    println!("Server listening on 0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_root(State(state): State<AppState>) -> String {
    let span = state.observability.start_span("handle_root");
    state.observability.record_counter("root_requests", 1).unwrap();
    
    let result = "Welcome to the example service!";
    
    state.observability.set_span_status(&span, SpanStatus::Ok, None);
    state.observability.end_span(span);
    
    result.to_string()
}

async fn handle_get_user(State(state): State<AppState>, Path(user_id): Path<String>) -> String {
    let span = state.observability.start_span("handle_get_user");
    state.observability.set_span_attribute(&span, "user_id", &user_id);
    state.observability.record_counter("user_requests", 1).unwrap();
    
    let labels = vec![("user_id", user_id.clone())];
    state.observability.record_counter_with_labels("user_id_requests", 1, &labels).unwrap();
    
    let result = format!("User details for ID: {}", user_id);
    
    state.observability.set_span_status(&span, SpanStatus::Ok, None);
    state.observability.end_span(span);
    
    result
}

async fn handle_health(State(state): State<AppState>) -> String {
    match state.observability.health_check() {
        Ok(true) => "Healthy".to_string(),
        _ => "Unhealthy".to_string(),
    }
}

async fn record_app_metrics(observability: Arc<observability::ObservabilityService>) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    
    // Simulated metrics values
    let mut memory_usage = 100.0;
    let mut cpu_usage = 5.0;
    
    loop {
        interval.tick().await;
        
        // Update simulated values
        memory_usage = (memory_usage + 10.0) % 1000.0;
        cpu_usage = (cpu_usage + 2.0) % 100.0;
        
        // Record metrics
        observability.record_gauge("memory_usage_mb", memory_usage).unwrap();
        observability.record_gauge("cpu_usage_percent", cpu_usage).unwrap();
        
        // Record uptime
        observability.record_counter("uptime_seconds", 5).unwrap();
    }
} 