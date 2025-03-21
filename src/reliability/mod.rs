//! Reliability middleware for enhancing application resilience
//!
//! This module provides a user-friendly interface for application-specific reliability features.
//! The core implementation is in the core/reliability module.

// User-extensible modules
pub mod circuit_breaker;
pub mod concurrency;
pub mod metrics;
pub mod rate_limit;
pub mod retry;

// Re-export core reliability functionality
pub use crate::core::reliability::*;

// This is a user-friendly reliability module where application-specific
// reliability features can be defined. Add your custom reliability
// features below.

// Example of defining application-specific reliability features:
//
// /// Custom circuit breaker for the user service
// pub fn create_user_service_circuit_breaker<S>(service: S) -> CircuitBreakerService<S> {
//     CircuitBreakerLayer::new(3, Duration::from_secs(30), 2)
//         .layer(service)
// }
//
// /// Custom rate limiter for the authentication endpoint
// pub fn create_auth_rate_limiter<S>(service: S) -> RateLimitService<S> {
//     RateLimitLayer::new(10, Duration::from_secs(60), true)
//         .layer(service)
// }
