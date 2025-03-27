# Core Reliability Module

This module provides reliability middleware components for enhancing application resilience.

## Features

- **Retries**: Automatically retry failed requests
- **Circuit Breaker**: Prevent cascading failures
- **Rate Limiting**: Control request rates
- **Concurrency Limiting**: Control concurrent request counts
- **Request Timeouts**: Ensure requests complete in a timely manner

## Usage

### Applying Reliability Features

The simplest way to enable reliability features is to use the `apply_reliability` function in your router setup:

```rust
use crate::core::reliability::apply_reliability;
use crate::core::config::app_config::ReliabilityConfig;

// In your router setup
let router = create_router();
let reliability_config = ReliabilityConfig::default();
let enhanced_router = apply_reliability(router, &reliability_config);
```

### Using Individual Reliability Components

You can also use the individual reliability components directly:

#### Circuit Breaker

```rust
use crate::core::reliability::CircuitBreakerLayer;
use std::time::Duration;

let circuit_breaker = CircuitBreakerLayer::new(
    Duration::from_secs(30), // Reset timeout
    2                       // Success threshold
);

let service = circuit_breaker.layer(my_service);
```