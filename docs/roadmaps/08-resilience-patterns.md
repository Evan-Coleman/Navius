# Essential Resilience Patterns

## Overview
A pragmatic approach to resilience for our Rust Axum backend that builds on existing implementations while ensuring production readiness, security, and integration with our AWS, Redis, Postgres, and Entra infrastructure.

## Current State
Our application already has implementations of circuit breakers, retry mechanisms, and timeouts, but these need assessment for production readiness and integration with our AWS infrastructure.

## Target State
A focused resilience system that:
- Ensures security during failure scenarios
- Makes existing resilience patterns production-ready
- Provides Axum middleware for easy integration
- Protects critical services (Postgres, Redis, external APIs)
- Includes essential monitoring for resilience health

## Implementation Progress Tracking

### Phase 1: Production-Ready Existing Patterns
1. **Circuit Breaker Hardening**
   - [ ] Audit existing circuit breaker implementation for production readiness
   - [ ] Add proper metrics and logging for circuit state changes
   - [ ] Implement secure failure handling for authentication services
   
   *Updated at: Not started*

2. **Retry Mechanism Enhancement**
   - [ ] Review and optimize existing retry logic
   - [ ] Add exponential backoff with jitter for AWS services
   - [ ] Create retry policies specific to Postgres and Redis operations
   
   *Updated at: Not started*

3. **Timeout Management**
   - [ ] Assess current timeout implementations
   - [ ] Add context propagation for nested timeouts
   - [ ] Create consistent timeout handling across all external calls
   
   *Updated at: Not started*

### Phase 2: Security-Focused Resilience
1. **Rate Limiting**
   - [ ] Implement IP-based rate limiting for public endpoints
   - [ ] Add tenant-based rate limiting using Redis
   - [ ] Create rate limit policies for authentication endpoints
   
   *Updated at: Not started*

2. **Secure Fallbacks**
   - [ ] Build secure default responses for authentication failures
   - [ ] Implement graceful degradation for non-critical features
   - [ ] Create fallback chains with security context preservation
   
   *Updated at: Not started*

3. **AWS Service Resilience**
   - [ ] Implement specific resilience patterns for AWS services
   - [ ] Add region failover capabilities for critical operations
   - [ ] Create secure credential refresh mechanism
   
   *Updated at: Not started*

### Phase 3: Axum Integration
1. **Resilience Middleware**
   - [ ] Create Axum middleware for core resilience patterns
   - [ ] Build route-specific resilience configuration
   - [ ] Implement middleware ordering for optimal resilience
   
   *Updated at: Not started*

2. **Observability**
   - [ ] Add logging for resilience-related events
   - [ ] Implement essential CloudWatch metrics for resilience patterns
   - [ ] Create simple alerting for resilience failures
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: Circuit Breaker Hardening

## Success Criteria
- All external service calls have appropriate resilience patterns
- Failures are handled gracefully without security implications
- System recovers automatically from temporary failures
- Critical paths have multiple layers of protection
- Resilience patterns are easily applied through middleware

## Implementation Notes
This approach focuses on making our existing resilience patterns production-ready while adding only essential new capabilities. We'll leverage Axum's middleware system for easy application of resilience patterns and integrate with our AWS, Redis, Postgres, and Entra infrastructure.

### Example Implementation

```rust
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Router,
};
use std::{sync::Arc, time::Duration};
use tokio::time::timeout;
use metrics::{counter, gauge};

// Enhanced circuit breaker with improved metrics and failure handling
#[derive(Clone)]
pub struct CircuitBreaker {
    state: Arc<tokio::sync::RwLock<CircuitState>>,
    config: CircuitBreakerConfig,
    name: String,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    failure_threshold: u32,
    success_threshold: u32,
    reset_timeout: Duration,
    half_open_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed { failures: u32 },
    Open { opened_at: std::time::Instant },
    HalfOpen { successes: u32 },
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let name = name.into();
        // Register metrics
        gauge!("circuit_breaker.state", 0.0, "name" => name.clone());
        counter!("circuit_breaker.success", 0, "name" => name.clone());
        counter!("circuit_breaker.failure", 0, "name" => name.clone());
        
        Self {
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed { failures: 0 })),
            config,
            name,
        }
    }
    
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if circuit is open
        {
            let state = self.state.read().await;
            match &*state {
                CircuitState::Open { opened_at } => {
                    let elapsed = opened_at.elapsed();
                    if elapsed < self.config.reset_timeout {
                        return Err(CircuitBreakerError::Open);
                    }
                    // Reset timeout has elapsed, will try half-open
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::HalfOpen { successes: 0 };
                    gauge!("circuit_breaker.state", 0.5, "name" => self.name.clone());
                    tracing::info!(circuit = self.name, "Circuit moved to half-open state");
                }
                CircuitState::HalfOpen { .. } => {
                    // Already in half-open, continue to test the service
                }
                CircuitState::Closed { .. } => {
                    // Circuit is closed, proceed normally
                }
            }
        }
        
        // Execute the function with timeout
        let result = match timeout(self.config.half_open_timeout, f()).await {
            Ok(inner_result) => inner_result,
            Err(_) => {
                self.record_failure().await;
                return Err(CircuitBreakerError::Timeout);
            }
        };
        
        // Process the result
        match result {
            Ok(value) => {
                self.record_success().await;
                Ok(value)
            }
            Err(err) => {
                self.record_failure().await;
                Err(CircuitBreakerError::Underlying(err))
            }
        }
    }
    
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        match &*state {
            CircuitState::HalfOpen { successes } => {
                let new_successes = successes + 1;
                if new_successes >= self.config.success_threshold {
                    *state = CircuitState::Closed { failures: 0 };
                    gauge!("circuit_breaker.state", 0.0, "name" => self.name.clone());
                    tracing::info!(circuit = self.name, "Circuit closed after successful tests");
                } else {
                    *state = CircuitState::HalfOpen { successes: new_successes };
                }
            }
            CircuitState::Closed { failures } => {
                *state = CircuitState::Closed { failures: 0 };
            }
            CircuitState::Open { .. } => {
                // Shouldn't happen but reset to closed just in case
                *state = CircuitState::Closed { failures: 0 };
                gauge!("circuit_breaker.state", 0.0, "name" => self.name.clone());
            }
        }
        counter!("circuit_breaker.success", 1, "name" => self.name.clone());
    }
    
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        match &*state {
            CircuitState::Closed { failures } => {
                let new_failures = failures + 1;
                if new_failures >= self.config.failure_threshold {
                    *state = CircuitState::Open { 
                        opened_at: std::time::Instant::now() 
                    };
                    gauge!("circuit_breaker.state", 1.0, "name" => self.name.clone());
                    tracing::warn!(circuit = self.name, "Circuit opened due to failures");
                } else {
                    *state = CircuitState::Closed { failures: new_failures };
                }
            }
            CircuitState::HalfOpen { .. } => {
                *state = CircuitState::Open { 
                    opened_at: std::time::Instant::now() 
                };
                gauge!("circuit_breaker.state", 1.0, "name" => self.name.clone());
                tracing::warn!(circuit = self.name, "Circuit reopened after failed test");
            }
            CircuitState::Open { opened_at } => {
                // Reset the timer
                *state = CircuitState::Open { 
                    opened_at: std::time::Instant::now() 
                };
            }
        }
        counter!("circuit_breaker.failure", 1, "name" => self.name.clone());
    }
}

// Error types for circuit breaker
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,           // Circuit is open, fast fail
    Timeout,        // Operation timed out
    Underlying(E),  // Underlying error from the service
}

// Axum middleware for rate limiting
pub async fn rate_limit<B>(
    State(rate_limiter): State<RateLimiter>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Get client IP from request
    let ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.ip())
        .unwrap_or_else(|| "0.0.0.0".parse().unwrap());
    
    // Get tenant ID from Entra identity if available
    let tenant_id = request
        .extensions()
        .get::<EntraIdentity>()
        .map(|identity| identity.tenant_id.as_str());
    
    // Try to acquire a rate limit token
    match rate_limiter.check_rate_limit(ip, tenant_id).await {
        Ok(()) => {
            // Proceed with the request
            Ok(next.run(request).await)
        }
        Err(RateLimitError::TooManyRequests) => {
            // Return 429 Too Many Requests
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        Err(RateLimitError::RedisError(_)) => {
            // Log the error but allow the request to proceed in case of Redis failure
            tracing::error!("Rate limiter Redis error - allowing request to proceed");
            Ok(next.run(request).await)
        }
    }
}

// Redis-based rate limiter with tenant isolation
#[derive(Clone)]
pub struct RateLimiter {
    redis: Arc<redis::Client>,
    ip_limit: u32,
    tenant_limit: u32,
    window_seconds: u32,
}

impl RateLimiter {
    pub fn new(
        redis_url: &str,
        ip_limit: u32,
        tenant_limit: u32,
        window_seconds: u32,
    ) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        
        Ok(Self {
            redis: Arc::new(client),
            ip_limit,
            tenant_limit,
            window_seconds,
        })
    }
    
    pub async fn check_rate_limit(
        &self,
        ip: IpAddr,
        tenant_id: Option<&str>,
    ) -> Result<(), RateLimitError> {
        let mut conn = self.redis.get_async_connection().await
            .map_err(RateLimitError::RedisError)?;
        
        // Check IP-based rate limit
        let ip_key = format!("ratelimit:ip:{}", ip);
        let ip_count: u32 = redis::cmd("INCR")
            .arg(&ip_key)
            .query_async(&mut conn)
            .await
            .map_err(RateLimitError::RedisError)?;
        
        // Set expiry if this is the first request in the window
        if ip_count == 1 {
            let _: () = redis::cmd("EXPIRE")
                .arg(&ip_key)
                .arg(self.window_seconds)
                .query_async(&mut conn)
                .await
                .map_err(RateLimitError::RedisError)?;
        }
        
        if ip_count > self.ip_limit {
            tracing::warn!(ip = %ip, count = ip_count, limit = self.ip_limit, "IP-based rate limit exceeded");
            return Err(RateLimitError::TooManyRequests);
        }
        
        // Check tenant-based rate limit if available
        if let Some(tenant) = tenant_id {
            let tenant_key = format!("ratelimit:tenant:{}", tenant);
            let tenant_count: u32 = redis::cmd("INCR")
                .arg(&tenant_key)
                .query_async(&mut conn)
                .await
                .map_err(RateLimitError::RedisError)?;
            
            if tenant_count == 1 {
                let _: () = redis::cmd("EXPIRE")
                    .arg(&tenant_key)
                    .arg(self.window_seconds)
                    .query_async(&mut conn)
                    .await
                    .map_err(RateLimitError::RedisError)?;
            }
            
            if tenant_count > self.tenant_limit {
                tracing::warn!(tenant = %tenant, count = tenant_count, limit = self.tenant_limit, "Tenant-based rate limit exceeded");
                return Err(RateLimitError::TooManyRequests);
            }
        }
        
        Ok(())
    }
}

#[derive(Debug)]
enum RateLimitError {
    TooManyRequests,
    RedisError(redis::RedisError),
}

// Configure resilience for the application
pub fn configure_resilience(app: Router) -> Router {
    // Create a rate limiter
    let rate_limiter = RateLimiter::new(
        "redis://localhost:6379", // Replace with actual configuration
        100,  // IP limit
        1000, // Tenant limit
        60,   // Window in seconds
    ).expect("Failed to create rate limiter");
    
    // Add middleware to the router
    app.layer(axum::middleware::from_fn_with_state(
        rate_limiter,
        rate_limit,
    ))
}

## References
- [AWS Builder's Library: Timeouts, retries, and backoff with jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)
- [Tokio](https://tokio.rs/)
- [governor crate](https://docs.rs/governor/latest/governor/) (for rate limiting)
- [circuit_breaker crate](https://docs.rs/circuit_breaker/latest/circuit_breaker/)
- [Axum middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
- [AWS Service Quotas](https://docs.aws.amazon.com/general/latest/gr/aws_service_limits.html) (for setting appropriate limits) 