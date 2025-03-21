# Reliability Extensions

This module provides a user-friendly interface to extend the core reliability features of the application. The core reliability implementation is in `src/core/reliability`.

## Structure

- `circuit_breaker.rs` - Define custom circuit breaker configurations
- `concurrency.rs` - Define custom concurrency limiting strategies
- `metrics.rs` - Define custom reliability metrics
- `rate_limit.rs` - Define custom rate limiting strategies
- `retry.rs` - Define custom retry strategies

## Usage

To create custom reliability components for your application features:

1. Choose the appropriate module for your reliability feature
2. Create a new function that applies the reliability middleware to a service
3. Use the new function in your router setup

### Example - Creating a Custom Circuit Breaker

```rust
// In src/reliability/circuit_breaker.rs
use std::time::Duration;
use crate::core::reliability::circuit_breaker::*;

/// Circuit breaker for user service API calls
pub fn user_service_circuit_breaker<S>(service: S) -> CircuitBreakerService<S> {
    CircuitBreakerLayer::new(
        3,                      // Allow 3 failures before tripping
        Duration::from_secs(30), // Wait 30 seconds before trying again
        2                       // Require 2 successes to close the circuit
    ).layer(service)
}
```

### Example - Using Custom Reliability in Router

```rust
// In your router module
use crate::reliability::circuit_breaker::user_service_circuit_breaker;

let user_service_router = Router::new()
    .route("/users", get(get_users))
    .route("/users/:id", get(get_user_by_id))
    .layer_fn(user_service_circuit_breaker);
```

## Integration with Core Reliability

The core reliability features can be automatically applied to your router using the `apply_reliability` function based on application configuration:

```rust
use crate::core::reliability::apply_reliability;
use crate::core::config::app_config::ReliabilityConfig;

// Get reliability config from application config
let reliability_config = app_config.reliability.clone();

// Apply reliability features to router
let router_with_reliability = apply_reliability(router, &reliability_config);
``` 