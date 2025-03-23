# Application Reliability

This directory contains user-facing reliability functionality for improving the resilience of the Navius application. Use this module to define custom reliability patterns and policies for your specific application needs.

## Usage

To use reliability features in your application code:

```rust
use crate::app::reliability;
use crate::core::error::Result;

// Using a standard retry policy
async fn fetch_with_retry() -> Result<String> {
    let retry_policy = reliability::create_db_retry_policy();
    
    let result = reliability::retry::execute_with_retry(
        || async {
            // Your operation that might fail temporarily
            fetch_data_from_database().await
        },
        retry_policy,
    ).await?;
    
    Ok(result)
}

// Using a circuit breaker
async fn call_external_service() -> Result<Response> {
    let circuit_breaker = reliability::circuit_breaker::CircuitBreaker::new(
        "external-service",
        reliability::circuit_breaker::CircuitBreakerSettings::default(),
    );
    
    let result = circuit_breaker.execute(|| async {
        // Call that might fail
        external_api_client.get("/some/endpoint").send().await
    }).await?;
    
    Ok(result)
}
```

## Extending Reliability Features

### Custom Retry Policies

Create custom retry policies for specific use cases:

```rust
// src/app/reliability/custom_retries.rs
use crate::core::reliability::retry::{RetryPolicy, RetryPolicyBuilder};
use std::time::Duration;

pub fn create_payment_gateway_retry_policy() -> impl RetryPolicy {
    RetryPolicyBuilder::default()
        .with_max_retries(5)
        .with_initial_backoff(Duration::from_millis(200))
        .with_backoff_factor(3.0)
        .with_jitter(true)
        .with_retry_if(|err: &reqwest::Error| err.is_timeout() || err.is_connect())
        .build()
}
```

### Custom Rate Limiters

Create custom rate limiters for specific endpoints:

```rust
// src/app/reliability/api_rate_limits.rs
use crate::core::reliability::rate_limit::{RateLimiter, TokenBucketLimiter};
use std::time::Duration;

pub fn create_api_rate_limiter() -> RateLimiter {
    TokenBucketLimiter::new(
        100,                     // 100 tokens (requests)
        10,                      // 10 tokens (requests) per refill
        Duration::from_secs(1),  // Refill every second
    )
}
```

## Best Practices

1. Define retry policies based on the specific failure modes of the target system
2. Use circuit breakers for external dependencies that might become unavailable
3. Apply rate limiting to protect both your system and downstream services
4. Configure appropriate timeouts for all external calls
5. Log reliability-related events (retries, circuit breaks, rate limits)
6. Include reliability metrics in your monitoring system
7. Test failure scenarios to ensure reliability features work as expected

## Core Reliability System

The core reliability system is provided by `crate::core::reliability` and includes:

- Retry mechanisms with backoff strategies
- Circuit breakers to prevent cascading failures
- Rate limiters to control resource usage
- Concurrency controls for managing parallel operations
- Reliability metrics for monitoring

Do not modify the core reliability system directly. Instead, use this directory to extend and customize reliability features for your specific application needs. 