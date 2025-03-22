# Essential Resilience Patterns

## Overview
A pragmatic approach to resilience for Navius that builds on existing implementations while ensuring production readiness, security, and integration with our infrastructure.

## Current State
Our application already has implementations of circuit breakers, retry mechanisms, and timeouts, but these need assessment for production readiness and integration with our infrastructure.

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
   - [ ] Add exponential backoff with jitter for external services
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

3. **External Service Resilience**
   - [ ] Implement specific resilience patterns for external services
   - [ ] Add failover capabilities for critical operations
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
   - [ ] Implement essential metrics for resilience patterns
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
This approach focuses on making our existing resilience patterns production-ready while adding only essential new capabilities. We'll leverage Axum's middleware system for easy application of resilience patterns.

Note: AWS-specific monitoring, authentication with Entra, and cloud service resilience patterns are covered in the AWS Integration roadmap.

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
    name: String,
    state: Arc<tokio::sync::RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    reset_timeout: Duration,
}

#[derive(Clone, Debug, PartialEq)]
enum CircuitState {
    Closed,
    Open(std::time::Instant),
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(name: &str, failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            name: name.to_string(),
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            failure_threshold,
            success_threshold: 2,
            reset_timeout,
        }
    }
    
    // Execute operation with circuit breaker protection
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, ResilienceError<E>>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let current_state = {
            let state = self.state.read().await;
            match *state {
                CircuitState::Open(opened_at) => {
                    if opened_at.elapsed() > self.reset_timeout {
                        // Reset timeout has passed, try half-open
                        drop(state);
                        let mut state = self.state.write().await;
                        *state = CircuitState::HalfOpen;
                        CircuitState::HalfOpen
                    } else {
                        // Still open, fast fail
                        counter!("circuit_breaker.fast_fail", "name" => self.name.clone()).increment(1);
                        return Err(ResilienceError::CircuitOpen);
                    }
                }
                state => state,
            }
        };
        
        // Track successful executions in half-open state
        let mut success_counter = 0;
        
        // Execute the operation
        match operation().await {
            Ok(result) => {
                // Handle success based on state
                if current_state == CircuitState::HalfOpen {
                    let mut state = self.state.write().await;
                    success_counter += 1;
                    
                    if success_counter >= self.success_threshold {
                        // Enough successes, close the circuit
                        *state = CircuitState::Closed;
                        counter!("circuit_breaker.closed", "name" => self.name.clone()).increment(1);
                    }
                }
                
                Ok(result)
            }
            Err(err) => {
                // Handle error based on state
                match current_state {
                    CircuitState::Closed => {
                        let mut state = self.state.write().await;
                        counter!("circuit_breaker.failure", "name" => self.name.clone()).increment(1);
                        
                        // Increment failure counter in metrics
                        if counter!("circuit_breaker.failure_count", "name" => self.name.clone()).increment(1) >= self.failure_threshold {
                            // Too many failures, open the circuit
                            *state = CircuitState::Open(std::time::Instant::now());
                            counter!("circuit_breaker.opened", "name" => self.name.clone()).increment(1);
                        }
                    }
                    CircuitState::HalfOpen => {
                        // Any failure in half-open state opens the circuit again
                        let mut state = self.state.write().await;
                        *state = CircuitState::Open(std::time::Instant::now());
                        counter!("circuit_breaker.reopened", "name" => self.name.clone()).increment(1);
                    }
                    _ => {}
                }
                
                Err(ResilienceError::OperationFailed(Box::new(err)))
            }
        }
    }
}

// Middleware for applying circuit breaker to specific routes
pub fn with_circuit_breaker(circuit_breaker: CircuitBreaker) -> axum::middleware::from_fn_with_state_arc<CircuitBreaker> {
    axum::middleware::from_fn_with_state(circuit_breaker, circuit_breaker_middleware)
}

async fn circuit_breaker_middleware<B>(
    State(circuit_breaker): State<CircuitBreaker>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Apply circuit breaker to the request
    let response = circuit_breaker
        .execute(|| {
            let request_future = next.run(request);
            Box::pin(async move {
                let response = request_future.await;
                
                // Consider 5xx errors as circuit breaker failures
                if response.status().is_server_error() {
                    Err(format!("Server error: {}", response.status()))
                } else {
                    Ok(response)
                }
            })
        })
        .await;
    
    match response {
        Ok(response) => Ok(response),
        Err(ResilienceError::CircuitOpen) => {
            // Return service unavailable when circuit is open
            Ok(StatusCode::SERVICE_UNAVAILABLE.into_response())
        }
        Err(_) => {
            // Other errors become internal server errors
            Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

// Enhanced retry mechanism with jitter
pub async fn with_retry<F, T, E>(
    operation: F,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> futures::future::BoxFuture<'static, Result<T, E>> + Send + Sync + Clone + 'static,
    T: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut attempts = 0;
    let mut delay = base_delay;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                attempts += 1;
                
                if attempts >= max_retries {
                    return Err(err);
                }
                
                // Exponential backoff with jitter
                let jitter = rand::random::<f64>() * 0.1 * delay.as_millis() as f64;
                delay = Duration::from_millis((delay.as_millis() as f64 * 1.5 + jitter) as u64);
                
                // Log retry attempt
                tracing::info!(
                    "Retrying operation (attempt {}/{}), delay: {:?}",
                    attempts + 1,
                    max_retries,
                    delay
                );
                
                // Wait before retrying
                tokio::time::sleep(delay).await;
            }
        }
    }
}

// Error types for resilience operations
#[derive(Debug)]
pub enum ResilienceError<E> {
    OperationFailed(Box<E>),
    CircuitOpen,
    Timeout,
    RateLimited,
}

// Configure application with resilience patterns
pub fn configure_resilience(app: Router) -> Router {
    // Create circuit breakers
    let db_circuit_breaker = CircuitBreaker::new("database", 5, Duration::from_secs(30));
    let auth_circuit_breaker = CircuitBreaker::new("auth", 3, Duration::from_secs(10));
    
    // Apply middleware to appropriate routes
    app.route_layer(with_circuit_breaker(db_circuit_breaker.clone()))
       .route(
           "/api/auth/*path",
           axum::routing::any(|| async {}).layer(with_circuit_breaker(auth_circuit_breaker)),
       )
       .with_state(db_circuit_breaker)
}

// Example usage of resilience patterns in an Axum handler
async fn resilient_api_call(
    State(db): State<Arc<dyn DbService>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Use timeout for external API call
    let api_result = match timeout(
        Duration::from_secs(2),
        with_retry(
            || {
                Box::pin(async {
                    // Make an external API call
                    let client = reqwest::Client::new();
                    let resp = client.get(&format!("https://api.example.com/data/{}", id))
                        .send()
                        .await?
                        .error_for_status()?
                        .json::<serde_json::Value>()
                        .await?;
                    
                    Ok::<_, reqwest::Error>(resp)
                })
            },
            3,
            Duration::from_millis(100),
        ),
    )
    .await
    {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(format!("API error: {}", e)),
        Err(_) => Err("API call timed out".to_string()),
    };
    
    match api_result {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(msg) => {
            // Use fallback strategy
            match db.get_cached_data(id).await {
                Ok(Some(cached)) => {
                    // Return stale data with warning
                    (
                        StatusCode::OK,
                        [("X-Data-Source", "cache")],
                        Json(cached),
                    )
                        .into_response()
                }
                _ => (StatusCode::SERVICE_UNAVAILABLE, msg).into_response(),
            }
        }
    }
}
```

## References
- [Resilience4j](https://github.com/resilience4j/resilience4j)
- [Axum Middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
- [Circuit Breaker Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker)
- [Backoff and Retry Strategies](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/) 