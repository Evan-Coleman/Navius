//! User-defined reliability metrics components
//!
//! This module allows you to define custom metrics and monitoring
//! for your application's reliability features. The core implementation
//! is provided by crate::core::reliability::metrics.

use crate::core::reliability::metrics::*;

// Example of custom reliability metrics:
//
// use metrics::{counter, gauge, histogram};
//
// /// Record a rate-limited request
// pub fn record_rate_limited_request(endpoint: &str) {
//     counter!("rate_limited_requests_total", 1, "endpoint" => endpoint.to_string());
// }
//
// /// Record circuit breaker state change
// pub fn record_circuit_breaker_state_change(name: &str, state: &str) {
//     counter!("circuit_breaker_state_changes_total", 1,
//         "name" => name.to_string(),
//         "state" => state.to_string()
//     );
// }
//
// /// Record retry attempt
// pub fn record_retry_attempt(endpoint: &str, attempt: u32) {
//     counter!("retry_attempts_total", 1,
//         "endpoint" => endpoint.to_string(),
//         "attempt" => attempt.to_string()
//     );
// }
