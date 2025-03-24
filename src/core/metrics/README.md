# Core Metrics Module

This module provides metrics collection and reporting functionality for the backend application.

## Features

- Prometheus-based metrics collection
- Metrics reporting endpoint
- Automatic metrics initialization

## Usage

### Initializing Metrics

To initialize the metrics system in your application:

```rust
use crate::core::metrics::init_metrics;

// In your app initialization
let metrics_handle = init_metrics();
```

### Recording Custom Metrics

You can record custom metrics using the metrics crate:

```rust
use metrics::{counter, gauge, histogram};

// Increment a counter
counter!("api_requests_total", 1, "endpoint" => "resources");

// Set a gauge value
gauge!("cache_size", 42.0, "type" => "resource");

// Record a histogram value
histogram!("request_duration_seconds", 0.157, "endpoint" => "resources");
```

### Exposing Metrics Endpoint

Add a metrics endpoint to your application router:

```rust
use axum::{Router, routing::get};
use std::sync::Arc;

// In your router setup
let app = Router::new()
    .route("/metrics", get(|| async move {
        metrics_handler(&metrics_handle)
    }));
```

## Integration with Actuator Routes

The metrics endpoint should typically be exposed via the actuator route group for monitoring purposes. 