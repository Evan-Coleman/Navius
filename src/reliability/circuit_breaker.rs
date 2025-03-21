//! User-defined circuit breaker components
//!
//! This module allows you to define custom circuit breaker configurations
//! for your application. The core implementation is provided by
//! crate::core::reliability::circuit_breaker.

use crate::core::reliability::circuit_breaker::*;
use std::time::Duration;

// Example of a custom circuit breaker for the user service:
//
// /// Circuit breaker configured for the user service
// pub fn user_service_circuit_breaker<S>(service: S) -> CircuitBreakerService<S> {
//     CircuitBreakerLayer::new(
//         5,                          // Allow 5 failures before tripping
//         Duration::from_secs(15),    // Wait 15 seconds before trying again
//         2                           // Require 2 successes to close the circuit
//     ).layer(service)
// }
//
// /// Circuit breaker with percentage-based failure detection
// pub fn auth_service_circuit_breaker<S>(service: S) -> CircuitBreakerService<S> {
//     CircuitBreakerLayer::new_with_config(
//         10,                         // Failure threshold (unused in percentage mode)
//         Duration::from_secs(30),    // Wait 30 seconds before trying again
//         3,                          // Require 3 successes to close the circuit
//         60,                         // 60 second rolling window
//         25,                         // 25% failure rate trips the circuit
//         false,                      // Use percentage-based detection (not consecutive)
//         vec![500, 502, 503]         // Status codes to count as failures
//     ).layer(service)
// }
