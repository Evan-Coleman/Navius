# Application Metrics

This directory contains user-facing metrics functionality for the Navius application. Use this module to define application-specific metrics that are not part of the core metrics system.

## Usage

To use metrics in your application code, import the metrics module:

```rust
use crate::app::metrics;
```

## Examples

### Recording Counter Metrics

```rust
// Record a user login event
pub fn record_user_login() {
    metrics::counter!("user_logins_total", 1);
}
```

### Recording Histogram Metrics

```rust
// Record API request duration
pub fn record_api_request_duration(endpoint: &str, duration_ms: f64) {
    metrics::histogram!(
        "api_request_duration_ms", 
        duration_ms, 
        "endpoint" => endpoint.to_string()
    );
}
```

### Recording Gauge Metrics

```rust
// Record cache hit rate
pub fn record_cache_hit_rate(rate: f64) {
    metrics::gauge!("cache_hit_rate", rate);
}
```

## Best Practices

1. Use descriptive metric names that follow a consistent naming pattern
2. Add relevant labels to metrics for better filtering and aggregation
3. Document each metric with a comment explaining what it measures
4. Group related metrics in their own module files
5. Prefix application-specific metrics with your application name
6. Use standard units (seconds for time, bytes for memory, etc.)

## Core Metrics System

The core metrics system is provided by `crate::core::metrics` and includes:

- Metrics initialization and setup
- Integration with the underlying metrics library
- Standard metrics for HTTP requests, system resources, and more
- Metrics export to various backends

Do not modify the core metrics system directly. Instead, use this directory to extend and customize metrics for your specific application needs. 